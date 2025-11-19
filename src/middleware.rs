use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

use crate::auth::{verify_token, Claims};
use crate::error::AppError;
use crate::pool_manager::{DatabaseConfig, POOL_MANAGER};

/// JWT 认证中间件
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    // 从 Authorization header 中提取 token
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(&h[7..])
            } else {
                None
            }
        })
        .ok_or_else(|| AppError::Unauthorized("缺少 Authorization header".to_string()))?;

    // 验证 token
    let claims = verify_token(token)?;

    // 将 claims 存入请求扩展，供后续处理器使用
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// 可选认证中间件（token 无效时不报错，仅在有效时添加用户信息）
pub async fn optional_auth_middleware(mut req: Request, next: Next) -> Response {
    // 尝试提取 token
    if let Some(token) = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(&h[7..])
            } else {
                None
            }
        })
    {
        // 尝试验证 token，如果成功则添加到扩展中
        if let Ok(claims) = verify_token(token) {
            req.extensions_mut().insert(claims);
        }
    }

    next.run(req).await
}

/// 动态数据库连接中间件
/// 从 X-Database-Id 请求头中获取数据库 ID，并从连接池管理器中获取对应的连接池
pub async fn dynamic_db_middleware(
    State(main_pool): State<PgPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 尝试从请求头中获取 X-Database-Id
    if let Some(db_id_str) = req.headers().get("X-Database-Id").and_then(|h| h.to_str().ok()) {
        tracing::info!("检测到 X-Database-Id 请求头: {}", db_id_str);
        if let Ok(database_id) = db_id_str.parse::<i32>() {
            // 从主数据库中获取连接配置
            
            let db_config_row = sqlx::query(
                r#"
                SELECT 
                    id, connection_name, db_host, db_port, db_name,
                    db_user, db_password_encrypted, max_connections, connection_timeout
                FROM management.tenant_databases
                WHERE id = $1 AND is_active = true
                "#,
            )
            .bind(database_id)
            .fetch_optional(&main_pool)
            .await
            .map_err(|e| AppError::Internal(format!("查询数据库配置失败: {}", e)))?;

            if let Some(row) = db_config_row {
                use sqlx::Row;
                
                // 简单解密（生产环境需要真正的解密）
                let encrypted_password: String = row.get("db_password_encrypted");
                let password = encrypted_password.trim_start_matches("ENCRYPTED:");
                
                let config = DatabaseConfig {
                    id: row.get("id"),
                    host: row.get("db_host"),
                    port: row.get("db_port"),
                    database: row.get("db_name"),
                    username: row.get("db_user"),
                    password: password.to_string(),
                    max_connections: row.get::<Option<i32>, _>("max_connections").unwrap_or(10) as u32,
                    connection_timeout: row.get::<Option<i32>, _>("connection_timeout").unwrap_or(30) as u64,
                };

                // 获取或创建连接池
                let pool = POOL_MANAGER.get_or_create_pool(config).await?;
                
                tracing::info!("成功切换到数据库连接 ID: {}", database_id);
                // 将动态连接池存入请求扩展
                req.extensions_mut().insert(pool);
            } else {
                tracing::warn!("未找到数据库连接配置: ID={}", database_id);
            }
        } else {
            tracing::warn!("无效的数据库 ID 格式: {}", db_id_str);
        }
    } else {
        tracing::debug!("未提供 X-Database-Id 请求头，使用默认连接池");
    }

    Ok(next.run(req).await)
}

/// 从请求中获取当前用户信息
pub fn get_current_user(req: &Request) -> Option<&Claims> {
    req.extensions().get::<Claims>()
}

/// 检查用户是否有指定角色
pub fn has_role(claims: &Claims, required_role: &str) -> bool {
    claims.role == required_role || claims.role == "admin" // admin 拥有所有权限
}

/// 角色检查中间件工厂
pub fn require_role(
    required_role: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>,
> + Clone {
    move |req: Request, next: Next| {
        Box::pin(async move {
            // 从扩展中获取 claims
            let claims = req
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| AppError::Unauthorized("未认证".to_string()))?;

            // 检查角色
            if !has_role(claims, required_role) {
                return Err(AppError::Forbidden(format!(
                    "需要 {} 角色权限",
                    required_role
                )));
            }

            Ok(next.run(req).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Claims;

    #[test]
    fn test_has_role() {
        let claims = Claims {
            sub: "123".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            exp: 9999999999,
            iat: 0,
        };

        assert!(has_role(&claims, "user"));
        assert!(!has_role(&claims, "admin"));

        let admin_claims = Claims {
            role: "admin".to_string(),
            ..claims
        };

        assert!(has_role(&admin_claims, "user"));
        assert!(has_role(&admin_claims, "admin"));
    }
}

