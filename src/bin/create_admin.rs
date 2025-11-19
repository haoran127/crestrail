use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取数据库 URL
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/crestrail".to_string());

    println!("🔗 连接数据库: {}", database_url);

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // 管理员信息
    let username = "admin";
    let email = "admin@example.com";
    let password = "Admin123";
    
    // 使用 bcrypt 哈希密码（cost=12）
    let password_hash = bcrypt::hash(password, 12)?;
    
    println!("👤 创建管理员账户:");
    println!("   邮箱: {}", email);
    println!("   密码: {}", password);
    
    // 检查用户是否已存在
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&pool)
        .await?;
    
    if let Some(row) = existing {
        let user_id: i32 = row.try_get("id")?;
        println!("⚠️  用户已存在，更新密码和权限...");
        
        sqlx::query("UPDATE users SET password_hash = $1, is_superadmin = true, role = 'admin', username = $2 WHERE email = $3")
            .bind(&password_hash)
            .bind(username)
            .bind(email)
            .execute(&pool)
            .await?;
        
        println!("✅ 用户已更新 (ID: {})", user_id);
    } else {
        println!("📝 创建新用户...");
        
        let row = sqlx::query("INSERT INTO users (username, email, password_hash, role, is_superadmin) VALUES ($1, $2, $3, 'admin', true) RETURNING id")
            .bind(username)
            .bind(email)
            .bind(&password_hash)
            .fetch_one(&pool)
            .await?;
        
        let user_id: i32 = row.try_get("id")?;
        println!("✅ 用户已创建 (ID: {})", user_id);
    }
    
    println!("\n🎉 超级管理员账户已就绪！");
    println!("   邮箱: {}", email);
    println!("   密码: {}", password);
    
    Ok(())
}

