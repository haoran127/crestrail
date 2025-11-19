use crate::auth::Claims;
use crate::error::{AppError, Result};
use crate::tenant_models::*;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

/// 检查是否为超级管理员
fn require_super_admin(claims: &Claims) -> Result<()> {
    if claims.role != "super_admin" {
        return Err(AppError::Unauthorized(
            "需要超级管理员权限".to_string(),
        ));
    }
    Ok(())
}

/// 检查是否为超级管理员或租户管理员
fn require_admin(claims: &Claims) -> Result<()> {
    if claims.role != "super_admin" && claims.role != "tenant_admin" {
        return Err(AppError::Unauthorized("需要管理员权限".to_string()));
    }
    Ok(())
}

// ==================== 租户管理 ====================

/// 创建租户请求
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    pub contact_email: Option<String>,
}

/// 租户列表响应
#[derive(Debug, Serialize)]
pub struct TenantListItem {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub contact_email: Option<String>,
    pub user_count: i64,
    pub database_count: i64,
    pub created_at: String,
}

/// GET /api/admin/tenants - 获取所有租户（仅超管）
pub async fn list_tenants(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<TenantListItem>>> {
    require_super_admin(&claims)?;

    let tenants = sqlx::query(
        r#"
        SELECT 
            t.id, t.name, t.slug, t.status, t.contact_email, t.created_at,
            COUNT(DISTINCT ut.user_id) as user_count,
            COUNT(DISTINCT td.id) as database_count
        FROM management.tenants t
        LEFT JOIN management.user_tenants ut ON ut.tenant_id = t.id AND ut.is_active = true
        LEFT JOIN management.tenant_databases td ON td.tenant_id = t.id AND td.is_active = true
        GROUP BY t.id, t.name, t.slug, t.status, t.contact_email, t.created_at
        ORDER BY t.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let result: Vec<TenantListItem> = tenants
        .iter()
        .map(|row| TenantListItem {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            status: row.get("status"),
            contact_email: row.get("contact_email"),
            user_count: row.get("user_count"),
            database_count: row.get("database_count"),
            created_at: row.get::<chrono::NaiveDateTime, _>("created_at").to_string(),
        })
        .collect();

    Ok(Json(result))
}

/// POST /api/admin/tenants - 创建租户（仅超管）
pub async fn create_tenant(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTenantRequest>,
) -> Result<Json<Tenant>> {
    require_super_admin(&claims)?;

    // 检查 slug 是否已存在
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM management.tenants WHERE slug = $1)",
    )
    .bind(&req.slug)
    .fetch_one(&pool)
    .await?;

    if existing {
        return Err(AppError::InvalidQuery(
            "租户标识 (slug) 已存在".to_string(),
        ));
    }

    let tenant = sqlx::query_as::<_, Tenant>(
        r#"
        INSERT INTO management.tenants (name, slug, contact_email)
        VALUES ($1, $2, $3)
        RETURNING id, name, slug, status, contact_email
        "#,
    )
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&req.contact_email)
    .fetch_one(&pool)
    .await?;

    tracing::info!(
        "超管 {} 创建了新租户: {} ({})",
        claims.email,
        tenant.name,
        tenant.slug
    );

    Ok(Json(tenant))
}

/// PUT /api/admin/tenants/:tenant_id/status - 更新租户状态（仅超管）
#[derive(Debug, Deserialize)]
pub struct UpdateTenantStatusRequest {
    pub status: String, // active, suspended, deleted
}

pub async fn update_tenant_status(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<i32>,
    Json(req): Json<UpdateTenantStatusRequest>,
) -> Result<Json<Tenant>> {
    require_super_admin(&claims)?;

    // 验证 status 值
    if !["active", "suspended", "deleted"].contains(&req.status.as_str()) {
        return Err(AppError::InvalidQuery(
            "无效的状态值".to_string(),
        ));
    }

    let tenant = sqlx::query_as::<_, Tenant>(
        r#"
        UPDATE management.tenants
        SET status = $1
        WHERE id = $2
        RETURNING id, name, slug, status, contact_email
        "#,
    )
    .bind(&req.status)
    .bind(tenant_id)
    .fetch_one(&pool)
    .await?;

    tracing::info!(
        "超管 {} 将租户 {} 的状态更新为 {}",
        claims.email,
        tenant.name,
        req.status
    );

    Ok(Json(tenant))
}

// ==================== 用户管理 ====================

/// 用户信息（包含角色）
#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub tenant_count: i64,
    pub created_at: String,
}

/// GET /api/admin/users - 获取所有用户（仅超管）
pub async fn list_users(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<UserListItem>>> {
    require_super_admin(&claims)?;

    let users = sqlx::query(
        r#"
        SELECT 
            u.id, u.username, u.email, u.role, u.created_at,
            COUNT(DISTINCT ut.tenant_id) as tenant_count
        FROM users u
        LEFT JOIN management.user_tenants ut ON ut.user_id = u.id AND ut.is_active = true
        GROUP BY u.id, u.username, u.email, u.role, u.created_at
        ORDER BY u.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let result: Vec<UserListItem> = users
        .iter()
        .map(|row| UserListItem {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            role: row.get("role"),
            tenant_count: row.get("tenant_count"),
            created_at: row.get::<chrono::NaiveDateTime, _>("created_at").to_string(),
        })
        .collect();

    Ok(Json(result))
}

/// 为租户添加用户请求
#[derive(Debug, Deserialize)]
pub struct AddUserToTenantRequest {
    pub user_id: i32,
    pub tenant_id: i32,
    pub role: String, // owner, admin, member, viewer
}

/// POST /api/admin/tenant-users - 为租户添加用户（仅超管）
pub async fn add_user_to_tenant(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AddUserToTenantRequest>,
) -> Result<Json<serde_json::Value>> {
    require_super_admin(&claims)?;

    // 验证角色
    if !["owner", "admin", "member", "viewer"].contains(&req.role.as_str()) {
        return Err(AppError::InvalidQuery("无效的角色".to_string()));
    }

    // 检查用户是否存在
    let user_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
    )
    .bind(req.user_id)
    .fetch_one(&pool)
    .await?;

    if !user_exists {
        return Err(AppError::NotFound("用户不存在".to_string()));
    }

    // 检查租户是否存在
    let tenant_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM management.tenants WHERE id = $1)",
    )
    .bind(req.tenant_id)
    .fetch_one(&pool)
    .await?;

    if !tenant_exists {
        return Err(AppError::NotFound("租户不存在".to_string()));
    }

    // 添加用户到租户（如果已存在则更新）
    sqlx::query(
        r#"
        INSERT INTO management.user_tenants (user_id, tenant_id, role, is_active)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (user_id, tenant_id) 
        DO UPDATE SET role = $3, is_active = true
        "#,
    )
    .bind(req.user_id)
    .bind(req.tenant_id)
    .bind(&req.role)
    .execute(&pool)
    .await?;

    tracing::info!(
        "超管 {} 将用户 {} 添加到租户 {} (角色: {})",
        claims.email,
        req.user_id,
        req.tenant_id,
        req.role
    );

    Ok(Json(serde_json::json!({
        "message": "用户添加成功",
        "user_id": req.user_id,
        "tenant_id": req.tenant_id,
        "role": req.role
    })))
}

/// GET /api/admin/tenants/:tenant_id/users - 获取租户的所有用户（仅超管）
pub async fn list_tenant_users(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<i32>,
) -> Result<Json<Vec<serde_json::Value>>> {
    require_super_admin(&claims)?;

    let users = sqlx::query(
        r#"
        SELECT 
            u.id, u.username, u.email, u.role as user_role,
            ut.role as tenant_role, ut.is_active, ut.created_at
        FROM management.user_tenants ut
        JOIN users u ON u.id = ut.user_id
        WHERE ut.tenant_id = $1
        ORDER BY ut.created_at DESC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(&pool)
    .await?;

    let result: Vec<serde_json::Value> = users
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "username": row.get::<String, _>("username"),
                "email": row.get::<String, _>("email"),
                "user_role": row.get::<String, _>("user_role"),
                "tenant_role": row.get::<String, _>("tenant_role"),
                "is_active": row.get::<bool, _>("is_active"),
                "created_at": row.get::<chrono::NaiveDateTime, _>("created_at").to_string(),
            })
        })
        .collect();

    Ok(Json(result))
}

/// DELETE /api/admin/tenant-users/:user_id/:tenant_id - 从租户移除用户（仅超管）
pub async fn remove_user_from_tenant(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
    Path((user_id, tenant_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>> {
    require_super_admin(&claims)?;

    sqlx::query(
        "UPDATE management.user_tenants SET is_active = false WHERE user_id = $1 AND tenant_id = $2",
    )
    .bind(user_id)
    .bind(tenant_id)
    .execute(&pool)
    .await?;

    tracing::info!(
        "超管 {} 将用户 {} 从租户 {} 移除",
        claims.email,
        user_id,
        tenant_id
    );

    Ok(Json(serde_json::json!({
        "message": "用户移除成功"
    })))
}

// ==================== 系统统计 ====================

/// 系统统计信息
#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_users: i64,
    pub total_tenants: i64,
    pub active_tenants: i64,
    pub total_databases: i64,
    pub super_admins: i64,
}

/// GET /api/admin/stats - 获取系统统计信息（仅超管）
pub async fn get_system_stats(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<SystemStats>> {
    require_super_admin(&claims)?;

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    let total_tenants = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM management.tenants")
        .fetch_one(&pool)
        .await?;

    let active_tenants = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM management.tenants WHERE status = 'active'",
    )
    .fetch_one(&pool)
    .await?;

    let total_databases = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM management.tenant_databases WHERE is_active = true",
    )
    .fetch_one(&pool)
    .await?;

    let super_admins = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE role = 'super_admin'",
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(SystemStats {
        total_users,
        total_tenants,
        active_tenants,
        total_databases,
        super_admins,
    }))
}
