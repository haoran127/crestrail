use sqlx::postgres::PgPoolOptions;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("🔌 连接到数据库: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✅ 数据库连接成功");

    let migration_sql = fs::read_to_string("migrations/003_create_management_schema.sql")?;

    println!("📝 创建多租户管理架构...");

    sqlx::raw_sql(&migration_sql)
        .execute(&pool)
        .await?;

    println!("✅ 多租户管理架构创建完成！");
    
    // 添加角色字段到 users 表
    println!("\n📝 配置用户角色系统...");
    
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
            ADD COLUMN role VARCHAR(50) DEFAULT 'user'
            "#
        )
        .execute(&pool)
        .await?;
        println!("✅ Role 字段添加成功");
    } else {
        println!("ℹ️  Role 字段已存在");
    }

    // 设置 admin 为超级管理员
    let admin_updated = sqlx::query(
        r#"
        UPDATE users 
        SET role = 'super_admin' 
        WHERE username = 'admin' OR email = 'admin@example.com'
        RETURNING id, username, email, role
        "#
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(row) = admin_updated {
        let username: String = row.get("username");
        let email: String = row.get("email");
        let role: String = row.get("role");
        println!("✅ 超级管理员设置成功:");
        println!("   用户名: {}", username);
        println!("   邮箱: {}", email);
        println!("   角色: {}", role);
    } else {
        println!("⚠️  警告: 未找到 admin 用户");
        println!("   请先注册一个账号，然后手动执行:");
        println!("   UPDATE users SET role = 'super_admin' WHERE username = 'your_username';");
    }
    
    println!();
    println!("📊 创建的管理表:");
    println!("   - management.tenants (租户)");
    println!("   - management.tenant_databases (数据库连接配置)");
    println!("   - management.tenant_schemas (业务 Schema)");
    println!("   - management.user_tenants (用户-租户关联)");
    println!("   - management.connection_access_logs (访问日志)");
    println!();
    println!("👥 用户角色:");
    println!("   - super_admin: 超级管理员（管理所有租户）");
    println!("   - tenant_admin: 租户管理员（管理自己的租户）");
    println!("   - user: 普通用户（只能访问被授权的数据）");
    println!();
    println!("🎨 查询示例:");
    println!("   SELECT * FROM management.v_user_connections WHERE username = 'admin';");

    Ok(())
}

