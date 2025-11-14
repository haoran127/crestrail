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
    let migration_sql = fs::read_to_string("migrations/002_create_example_tables.sql")?;

    println!("📝 创建示例表和外键关系...");

    // 执行迁移
    sqlx::raw_sql(&migration_sql)
        .execute(&pool)
        .await?;

    println!("✅ 示例表创建完成！");
    println!();
    println!("📊 创建的表:");
    println!("   - categories (产品分类)");
    println!("   - products (产品) → 外键到 categories");
    println!("   - orders (订单) → 外键到 users");
    println!("   - order_items (订单明细) → 外键到 orders, products");
    println!("   - user_addresses (用户地址) → 外键到 users");
    println!("   - product_reviews (产品评论) → 外键到 users, products");
    println!();
    println!("🎨 现在可以在 ER 图中看到表之间的关系了！");

    Ok(())
}

