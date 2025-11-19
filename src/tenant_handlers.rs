use crate::auth::Claims;
use crate::error::Result;
use crate::pool_manager::{DatabaseConfig, POOL_MANAGER};
use crate::tenant_models::*;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde_json::json;
use sqlx::{PgPool, Row};

/// 简单的密码加密/解密（生产环境需要使用真正的加密）
fn encrypt_password(password: &str) -> String {
    format!("ENCRYPTED:{}", password)
}

fn decrypt_password(encrypted: &str) -> String {
    encrypted.trim_start_matches("ENCRYPTED:").to_string()
}

/// GET /api/tenants/my-connections - 获取当前用户可访问的所有连接
pub async fn get_my_connections(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<UserConnection>>> {
    let user_id = claims.sub;
    
    // 检查用户是否是超级管理员
    let is_superadmin = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_superadmin, false) FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    
    let connections = if is_superadmin {
        // 超管可以看到所有连接
        sqlx::query_as::<_, UserConnection>(
            r#"
            SELECT DISTINCT
                $1 AS user_id,
                u.username,
                t.id AS tenant_id,
                t.name AS tenant_name,
                td.id AS database_id,
                td.connection_name,
                td.db_host,
                td.db_port,
                td.db_name,
                td.is_primary,
                'superadmin' AS user_role
            FROM management.tenants t
            CROSS JOIN users u
            JOIN management.tenant_databases td ON td.tenant_id = t.id AND td.is_active = true
            WHERE u.id = $1 AND t.status = 'active'
            ORDER BY t.name, td.is_primary DESC, td.connection_name
            "#,
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await?
    } else {
        // 普通用户只能看到自己有权限的连接
        sqlx::query_as::<_, UserConnection>(
            r#"
            SELECT 
                user_id,
                username,
                tenant_id,
                tenant_name,
                database_id,
                connection_name,
                db_host,
                db_port,
                db_name,
                is_primary,
                user_role
            FROM management.v_user_connections
            WHERE user_id = $1
            ORDER BY tenant_name, is_primary DESC, connection_name
            "#,
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await?
    };

    Ok(Json(connections))
}

/// GET /api/tenants/:tenant_id/schemas - 获取租户的所有业务 Schema
pub async fn get_tenant_schemas(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<i32>,
) -> Result<Json<Vec<TenantSchema>>> {
    let user_id = claims.sub; // claims.sub 现在是 i32 类型
    
    // 验证用户是否有权限访问该租户
    let has_access = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM management.user_tenants
            WHERE user_id = $1 AND tenant_id = $2 AND is_active = true
        )
        "#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .fetch_one(&pool)
    .await?;

    if !has_access {
        return Err(crate::error::AppError::Unauthorized(
            "无权访问该租户".to_string(),
        ));
    }

    let schemas = sqlx::query_as::<_, TenantSchema>(
        r#"
        SELECT 
            id, tenant_id, database_id, schema_name, 
            business_type, display_name, description, is_active
        FROM management.tenant_schemas
        WHERE tenant_id = $1 AND is_active = true
        ORDER BY display_name
        "#,
    )
    .bind(tenant_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(schemas))
}

/// POST /api/tenants/test-connection - 测试数据库连接
pub async fn test_connection(
    Json(req): Json<TestConnectionRequest>,
) -> Result<Json<TestConnectionResponse>> {
    let connection_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        req.username, req.password, req.host, req.port, req.database
    );

    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&connection_url)
        .await
    {
        Ok(pool) => {
            // 获取服务器版本
            let version = sqlx::query_scalar::<_, String>("SELECT version()")
                .fetch_one(&pool)
                .await
                .ok();

            pool.close().await;

            Ok(Json(TestConnectionResponse {
                success: true,
                message: "连接成功".to_string(),
                server_version: version,
            }))
        }
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            message: format!("连接失败: {}", e),
            server_version: None,
        })),
    }
}

/// POST /api/tenants/connections - 创建新的数据库连接
pub async fn create_database_connection(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateDatabaseConnectionRequest>,
) -> Result<Json<TenantDatabase>> {
    let user_id = claims.sub; // claims.sub 现在是 i32 类型
    
    // 验证用户是否有权限管理该租户
    let user_role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role FROM management.user_tenants
        WHERE user_id = $1 AND tenant_id = $2 AND is_active = true
        "#,
    )
    .bind(&user_id)
    .bind(req.tenant_id)
    .fetch_optional(&pool)
    .await?;

    match user_role.as_deref() {
        Some("owner") | Some("admin") => {}
        _ => {
            return Err(crate::error::AppError::Unauthorized(
                "只有租户 owner 或 admin 可以创建连接".to_string(),
            ))
        }
    }

    // 加密密码
    let encrypted_password = encrypt_password(&req.db_password);

    // 插入数据库连接配置
    let db_connection = sqlx::query_as::<_, TenantDatabase>(
        r#"
        INSERT INTO management.tenant_databases 
        (tenant_id, connection_name, db_host, db_port, db_name, db_user, 
         db_password_encrypted, is_primary, max_connections, connection_timeout)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, tenant_id, connection_name, db_host, db_port, db_name, 
                  db_user, db_password_encrypted, is_primary, is_active, 
                  max_connections, connection_timeout
        "#,
    )
    .bind(req.tenant_id)
    .bind(&req.connection_name)
    .bind(&req.db_host)
    .bind(req.db_port)
    .bind(&req.db_name)
    .bind(&req.db_user)
    .bind(&encrypted_password)
    .bind(req.is_primary)
    .bind(req.max_connections.unwrap_or(10))
    .bind(req.connection_timeout.unwrap_or(30))
    .fetch_one(&pool)
    .await?;

    tracing::info!(
        "用户 {} 为租户 {} 创建了新连接: {}",
        user_id,
        req.tenant_id,
        req.connection_name
    );

    Ok(Json(db_connection))
}

/// POST /api/tenants/switch-connection - 切换到指定的数据库连接
pub async fn switch_connection(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SwitchConnectionRequest>,
) -> Result<Json<SwitchConnectionResponse>> {
    let user_id = claims.sub; // claims.sub 现在是 i32 类型
    
    // 获取连接配置
    let db_config = sqlx::query(
        r#"
        SELECT 
            td.id, td.connection_name, td.db_host, td.db_port, td.db_name,
            td.db_user, td.db_password_encrypted, td.max_connections, td.connection_timeout
        FROM management.tenant_databases td
        JOIN management.user_tenants ut ON ut.tenant_id = td.tenant_id
        WHERE td.id = $1 AND ut.user_id = $2 AND td.is_active = true AND ut.is_active = true
        "#,
    )
    .bind(req.database_id)
    .bind(&user_id)
    .fetch_optional(&pool)
    .await?;

    let row = db_config.ok_or_else(|| {
        crate::error::AppError::NotFound("数据库连接不存在或无权访问".to_string())
    })?;

    let config = DatabaseConfig {
        id: row.get("id"),
        host: row.get("db_host"),
        port: row.get("db_port"),
        database: row.get("db_name"),
        username: row.get("db_user"),
        password: decrypt_password(row.get("db_password_encrypted")),
        max_connections: row.get::<Option<i32>, _>("max_connections").unwrap_or(10) as u32,
        connection_timeout: row
            .get::<Option<i32>, _>("connection_timeout")
            .unwrap_or(30) as u64,
    };

    // 创建或获取连接池
    let _pool = POOL_MANAGER.get_or_create_pool(config).await?;

    tracing::info!(
        "用户 {} 切换到数据库连接 {}",
        user_id,
        req.database_id
    );

    Ok(Json(SwitchConnectionResponse {
        success: true,
        database_id: req.database_id,
        connection_name: row.get("connection_name"),
        message: "连接切换成功".to_string(),
    }))
}

/// GET /api/tenants/pool-stats - 获取连接池统计信息（管理员功能）
pub async fn get_pool_stats(
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    // 这里可以添加管理员权限检查
    let active_pools = POOL_MANAGER.active_pools_count();
    
    let user_id = claims.sub; // claims.sub 现在是 i32 类型

    Ok(Json(json!({
        "active_pools": active_pools,
        "user_id": user_id,
    })))
}

/// GET /api/admin/tenants - 获取所有租户（超管专用）
pub async fn list_all_tenants(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let user_id = claims.sub;
    
    // 验证超管权限
    let is_superadmin = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_superadmin, false) FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    
    if !is_superadmin {
        return Err(crate::error::AppError::Unauthorized(
            "需要超级管理员权限".to_string(),
        ));
    }
    
    let tenants = sqlx::query(
        r#"
        SELECT 
            tenant_id,
            tenant_name,
            slug,
            status,
            contact_email,
            database_count,
            schema_count,
            user_count,
            users,
            created_at::TEXT as created_at
        FROM management.v_superadmin_dashboard
        "#,
    )
    .fetch_all(&pool)
    .await?;
    
    let result: Vec<serde_json::Value> = tenants
        .iter()
        .map(|row| {
            json!({
                "tenant_id": row.get::<i32, _>("tenant_id"),
                "tenant_name": row.get::<String, _>("tenant_name"),
                "slug": row.get::<String, _>("slug"),
                "status": row.get::<String, _>("status"),
                "contact_email": row.get::<Option<String>, _>("contact_email"),
                "database_count": row.get::<i64, _>("database_count"),
                "schema_count": row.get::<i64, _>("schema_count"),
                "user_count": row.get::<i64, _>("user_count"),
                "users": row.get::<Vec<String>, _>("users"),
                "created_at": row.get::<String, _>("created_at"),
            })
        })
        .collect();
    
    Ok(Json(result))
}

/// POST /api/admin/tenants - 创建新租户（超管专用）
pub async fn create_tenant(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let user_id = claims.sub;
    
    // 验证超管权限
    let is_superadmin = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_superadmin, false) FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    
    if !is_superadmin {
        return Err(crate::error::AppError::Unauthorized(
            "需要超级管理员权限".to_string(),
        ));
    }
    
    let name = req["name"].as_str().ok_or_else(|| {
        crate::error::AppError::InvalidQuery("缺少租户名称".to_string())
    })?;
    
    let slug = req["slug"].as_str().ok_or_else(|| {
        crate::error::AppError::InvalidQuery("缺少租户标识".to_string())
    })?;
    
    let contact_email = req["contact_email"].as_str();
    
    let tenant = sqlx::query(
        r#"
        INSERT INTO management.tenants (name, slug, contact_email, status)
        VALUES ($1, $2, $3, 'active')
        RETURNING id, name, slug, status, contact_email, created_at::TEXT as created_at
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(contact_email)
    .fetch_one(&pool)
    .await?;
    
    tracing::info!("超管 {} 创建了新租户: {}", user_id, name);
    
    Ok(Json(json!({
        "id": tenant.get::<i32, _>("id"),
        "name": tenant.get::<String, _>("name"),
        "slug": tenant.get::<String, _>("slug"),
        "status": tenant.get::<String, _>("status"),
        "contact_email": tenant.get::<Option<String>, _>("contact_email"),
        "created_at": tenant.get::<String, _>("created_at"),
    })))
}

/// GET /api/admin/users - 获取所有用户（超管专用）
pub async fn list_all_users(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let user_id = claims.sub;
    
    // 验证超管权限
    let is_superadmin = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_superadmin, false) FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    
    if !is_superadmin {
        return Err(crate::error::AppError::Unauthorized(
            "需要超级管理员权限".to_string(),
        ));
    }
    
    let users = sqlx::query(
        r#"
        SELECT 
            u.id,
            u.username,
            u.email,
            COALESCE(u.is_superadmin, false) AS is_superadmin,
            u.created_at::TEXT as created_at,
            ARRAY_AGG(
                DISTINCT jsonb_build_object(
                    'tenant_id', t.id,
                    'tenant_name', t.name,
                    'role', ut.role
                )
            ) FILTER (WHERE t.id IS NOT NULL) AS tenants
        FROM users u
        LEFT JOIN management.user_tenants ut ON ut.user_id = u.id AND ut.is_active = true
        LEFT JOIN management.tenants t ON t.id = ut.tenant_id
        GROUP BY u.id, u.username, u.email, u.is_superadmin, u.created_at
        ORDER BY u.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;
    
    let result: Vec<serde_json::Value> = users
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<i32, _>("id"),
                "username": row.get::<String, _>("username"),
                "email": row.get::<String, _>("email"),
                "is_superadmin": row.get::<bool, _>("is_superadmin"),
                "created_at": row.get::<String, _>("created_at"),
                "tenants": row.get::<Vec<serde_json::Value>, _>("tenants"),
            })
        })
        .collect();
    
    Ok(Json(result))
}

/// POST /api/admin/users/:user_id/assign-tenant - 将用户分配给租户（超管专用）
pub async fn assign_user_to_tenant(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(target_user_id): Path<i32>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let user_id = claims.sub;
    
    // 验证超管权限
    let is_superadmin = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_superadmin, false) FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    
    if !is_superadmin {
        return Err(crate::error::AppError::Unauthorized(
            "需要超级管理员权限".to_string(),
        ));
    }
    
    let tenant_id = req["tenant_id"].as_i64().ok_or_else(|| {
        crate::error::AppError::InvalidQuery("缺少租户ID".to_string())
    })? as i32;
    
    let role = req["role"].as_str().unwrap_or("member");
    
    sqlx::query(
        r#"
        INSERT INTO management.user_tenants (user_id, tenant_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, tenant_id) 
        DO UPDATE SET role = $3, is_active = true
        "#,
    )
    .bind(target_user_id)
    .bind(tenant_id)
    .bind(role)
    .execute(&pool)
    .await?;
    
    tracing::info!(
        "超管 {} 将用户 {} 分配给租户 {} (角色: {})",
        user_id,
        target_user_id,
        tenant_id,
        role
    );
    
    Ok(Json(json!({
        "success": true,
        "message": "用户已成功分配给租户",
    })))
}

