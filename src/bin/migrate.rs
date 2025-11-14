use sqlx::postgres::PgPoolOptions;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    println!("🔌 连接到数据库: {}", database_url);

    // 连接数据库
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✅ 数据库连接成功");

    // 读取迁移文件
    let migration_sql = fs::read_to_string("migrations/001_create_users_table.sql")?;

    println!("📝 执行迁移脚本...");

    // 执行迁移
    sqlx::raw_sql(&migration_sql)
        .execute(&pool)
        .await?;

    println!("✅ 迁移完成！");

    Ok(())
}

