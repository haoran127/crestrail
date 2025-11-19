use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

use crate::error::AppError;

// JWT 配置
static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET 未设置，使用默认值（不安全！）");
        "your-secret-key-change-this-in-production".to_string()
    })
});

static JWT_EXPIRATION: Lazy<i64> = Lazy::new(|| {
    env::var("JWT_EXPIRATION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24 * 3600) // 默认 24 小时
});

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,         // 用户 ID (改为 i32 类型)
    pub email: String,    // 用户邮箱
    pub role: String,     // 用户角色
    pub exp: i64,         // 过期时间（Unix 时间戳）
    pub iat: i64,         // 签发时间
}

impl Claims {
    /// 创建新的 Claims
    pub fn new(user_id: i32, email: String, role: String) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::seconds(*JWT_EXPIRATION)).timestamp();

        Self {
            sub: user_id,
            email,
            role,
            exp,
            iat: now.timestamp(),
        }
    }

    /// 检查 token 是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// 生成 JWT token
pub fn generate_token(user_id: i32, email: &str, role: &str) -> Result<String, AppError> {
    let claims = Claims::new(user_id, email.to_string(), role.to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("生成 token 失败: {}", e)))
}

/// 验证 JWT token
pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("Token 已过期".to_string())
        }
        jsonwebtoken::errors::ErrorKind::InvalidToken => {
            AppError::Unauthorized("无效的 token".to_string())
        }
        _ => AppError::Unauthorized(format!("Token 验证失败: {}", e)),
    })?;

    Ok(token_data.claims)
}

/// 哈希密码
pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(format!("密码验证失败: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let token = generate_token("123", "test@example.com", "user").unwrap();
        let claims = verify_token(&token).unwrap();

        assert_eq!(claims.sub, "123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_token("invalid.token.here");
        assert!(result.is_err());
    }
}

