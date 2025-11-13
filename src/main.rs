mod config;
mod db;
mod error;
mod handlers;
mod query_builder;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use config::Config;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,crestrail=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = Config::from_env()?;
    tracing::info!("配置加载成功");

    // 创建数据库连接池
    let pool = db::create_pool(&config.database_url).await?;

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        .route("/api/:schema/:table", get(handlers::get_records))
        .route("/api/:schema/:table", post(handlers::create_record))
        .route("/api/:schema/:table", patch(handlers::update_records))
        .route("/api/:schema/:table", delete(handlers::delete_records))
        .with_state(pool)
        .layer(cors);

    // 启动服务器
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("🚀 服务器启动在 http://{}", addr);
    tracing::info!("📡 API 端点: http://{}/api/:schema/:table", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

