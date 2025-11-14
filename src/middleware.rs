use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

use crate::auth::{verify_token, Claims};
use crate::error::AppError;

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

