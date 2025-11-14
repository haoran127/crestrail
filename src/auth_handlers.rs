use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use validator::Validate;

use crate::auth::{generate_token, hash_password, verify_password, Claims};
use crate::error::AppError;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserInfo};

/// 用户注册
pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::InvalidQuery(format!("验证失败: {}", e)))?;

    // 检查邮箱是否已存在
    let existing_user = sqlx::query!(
        r#"SELECT id FROM users WHERE email = $1"#,
        req.email
    )
    .fetch_optional(&pool)
    .await?;

    if existing_user.is_some() {
        return Err(AppError::InvalidQuery("邮箱已被注册".to_string()));
    }

    // 检查用户名是否已存在
    let existing_username = sqlx::query!(
        r#"SELECT id FROM users WHERE username = $1"#,
        req.username
    )
    .fetch_optional(&pool)
    .await?;

    if existing_username.is_some() {
        return Err(AppError::InvalidQuery("用户名已被使用".to_string()));
    }

    // 哈希密码
    let password_hash = hash_password(&req.password)?;

    // 插入新用户
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, 'user')
        RETURNING id, username, email, role, created_at
        "#,
        req.username,
        req.email,
        password_hash
    )
    .fetch_one(&pool)
    .await?;

    // 生成 token
    let token = generate_token(&user.id.to_string(), &user.email, &user.role)?;

    let user_info = UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_string(),
    };

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user_info,
        }),
    ))
}

/// 用户登录
pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::InvalidQuery(format!("验证失败: {}", e)))?;

    // 查询用户
    let user = sqlx::query!(
        r#"
        SELECT id, username, email, password_hash, role, created_at
        FROM users
        WHERE email = $1
        "#,
        req.email
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("邮箱或密码错误".to_string()))?;

    // 验证密码
    let password_valid = verify_password(&req.password, &user.password_hash)?;
    if !password_valid {
        return Err(AppError::Unauthorized("邮箱或密码错误".to_string()));
    }

    // 生成 token
    let token = generate_token(&user.id.to_string(), &user.email, &user.role)?;

    let user_info = UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_string(),
    };

    Ok(Json(AuthResponse {
        token,
        user: user_info,
    }))
}

/// 获取当前用户信息
pub async fn get_me(
    Extension(claims): Extension<Claims>,
    State(pool): State<PgPool>,
) -> Result<Json<UserInfo>, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("无效的用户 ID".to_string()))?;

    let user = sqlx::query!(
        r#"
        SELECT id, username, email, role, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("用户不存在".to_string()))?;

    Ok(Json(UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.to_string(),
    }))
}

/// 刷新 token
pub async fn refresh_token(
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, AppError> {
    // 生成新的 token
    let new_token = generate_token(&claims.sub, &claims.email, &claims.role)?;

    Ok(Json(json!({
        "token": new_token
    })))
}

/// 修改密码
#[derive(serde::Deserialize, validator::Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1))]
    pub old_password: String,

    #[validate(length(min = 8))]
    pub new_password: String,
}

pub async fn change_password(
    Extension(claims): Extension<Claims>,
    State(pool): State<PgPool>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    req.validate()
        .map_err(|e| AppError::InvalidQuery(format!("验证失败: {}", e)))?;

    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("无效的用户 ID".to_string()))?;

    // 获取用户当前密码
    let user = sqlx::query!(
        r#"SELECT password_hash FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("用户不存在".to_string()))?;

    // 验证旧密码
    let password_valid = verify_password(&req.old_password, &user.password_hash)?;
    if !password_valid {
        return Err(AppError::Unauthorized("旧密码错误".to_string()));
    }

    // 哈希新密码
    let new_password_hash = hash_password(&req.new_password)?;

    // 更新密码
    sqlx::query!(
        r#"UPDATE users SET password_hash = $1 WHERE id = $2"#,
        new_password_hash,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(Json(json!({
        "message": "密码修改成功"
    })))
}

