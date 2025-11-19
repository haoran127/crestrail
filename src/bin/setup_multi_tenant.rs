use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始配置多租户系统...\n");

    // 从环境变量获取数据库连接
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:123456@localhost/crestrail".to_string());

    println!("📦 连接数据库: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✅ 数据库连接成功\n");

    // 1. 创建 management schema
    println!("📋 步骤 1: 创建 management schema...");
    sqlx::query("CREATE SCHEMA IF NOT EXISTS management")
        .execute(&pool)
        .await?;
    println!("✅ Management schema 创建成功\n");

    // 2. 读取并执行迁移脚本
    println!("📋 步骤 2: 执行多租户表结构迁移...");
    let migration_sql = include_str!("../../migrations/003_create_management_schema.sql");
    
    // 分批执行 SQL（因为可能有多个语句）
    sqlx::query(migration_sql)
        .execute(&pool)
        .await?;
    
    println!("✅ 多租户表结构创建成功\n");

    // 3. 确保 admin 用户存在并设置为超管
    println!("📋 步骤 3: 设置超级管理员...");
    
    // 检查 users 表是否有 role 字段
    let has_role_column = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'users' AND column_name = 'role'
        )
        "#
    )
    .fetch_one(&pool)
    .await?;

    if !has_role_column {
        println!("⚙️  添加 role 字段到 users 表...");
        sqlx::query(
            r#"
            ALTER TABLE users 
            ADD COLUMN IF NOT EXISTS role VARCHAR(50) DEFAULT 'user'
            "#
        )
        .execute(&pool)
        .await?;
        println!("✅ Role 字段添加成功");
    }

    // 更新 admin 用户为超管
    let admin_updated = sqlx::query(
        r#"
        UPDATE users 
        SET role = 'super_admin' 
        WHERE username = 'admin' OR email = 'admin@example.com'
        RETURNING id, username, role
        "#
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(row) = admin_updated {
        let username: String = row.get("username");
        let role: String = row.get("role");
        println!("✅ 超级管理员设置成功: {} ({})", username, role);
    } else {
        println!("⚠️  警告: 未找到 admin 用户，请先注册");
    }

    println!("\n🎉 多租户系统配置完成！\n");
    println!("📝 接下来你可以：");
    println!("   1. 使用 admin 账号登录系统");
    println!("   2. 访问租户管理页面创建租户");
    println!("   3. 为租户配置数据库连接");
    println!("   4. 为租户添加用户\n");

    Ok(())
}

