mod admin_handlers;
mod auth;
mod auth_handlers;
mod config;
mod db;
mod error;
mod export_handlers;
mod handlers;
mod middleware;
mod models;
mod monitor_handlers;
mod pool_manager;
mod query_builder;
mod schema_handlers;
mod tenant_handlers;
mod tenant_models;
mod transaction;

use axum::{
    extract::State,
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Json, Router,
};
use config::Config;
use error::AppError;
use serde_json::Value;
use sqlx::PgPool;
use tower_http::{
    cors::{Any, CorsLayer},
};
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

    // 公开路由
    let public_routes = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .route("/auth/register", post(auth_handlers::register))
        .route("/auth/login", post(auth_handlers::login))
        .route("/transaction", post(transaction::execute_transaction))
        .route("/query", post(execute_sql_query));
        // 前端使用独立的 Next.js 服务器（端口 3001）

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/auth/me", get(auth_handlers::get_me))
        .route("/auth/refresh", post(auth_handlers::refresh_token))
        .route("/auth/change-password", post(auth_handlers::change_password))
        .layer(axum_middleware::from_fn(middleware::auth_middleware));

    // Schema 管理路由（支持动态数据库连接）
    let schema_routes = Router::new()
        .route("/api/schemas", get(schema_handlers::list_schemas))
        .route("/api/schema/:schema/tables", get(schema_handlers::list_tables))
        .route("/api/schema/:schema/table/:table/structure", get(schema_handlers::get_table_structure))
        .route("/api/schema/:schema/table/:table/relationships", get(schema_handlers::get_table_relationships))
        .layer(axum_middleware::from_fn_with_state(pool.clone(), middleware::dynamic_db_middleware)); // 添加动态数据库中间件

    // 导出路由
    let export_routes = Router::new()
        .route("/api/export/csv/:schema/:table", get(export_handlers::export_csv))
        .route("/api/export/json/:schema/:table", get(export_handlers::export_json))
        .route("/api/export/sql/csv", post(export_handlers::export_sql_csv));

    // 监控路由
    let monitor_routes = Router::new()
        .route("/api/monitor/stats", get(monitor_handlers::get_database_stats))
        .route("/api/monitor/tables", get(monitor_handlers::get_table_sizes))
        .route("/api/monitor/slow-queries", get(monitor_handlers::get_slow_queries))
        .route("/api/monitor/connections", get(monitor_handlers::get_active_connections));

    // 租户管理路由
    let tenant_routes = Router::new()
        .route("/api/tenants/my-connections", get(tenant_handlers::get_my_connections))
        .route("/api/tenants/:tenant_id/schemas", get(tenant_handlers::get_tenant_schemas))
        .route("/api/tenants/test-connection", post(tenant_handlers::test_connection))
        .route("/api/tenants/connections", post(tenant_handlers::create_database_connection))
        .route("/api/tenants/switch-connection", post(tenant_handlers::switch_connection))
        .route("/api/tenants/pool-stats", get(tenant_handlers::get_pool_stats))
        .layer(axum_middleware::from_fn(middleware::auth_middleware));
    
    // 超管租户管理路由（使用 tenant_handlers）
    let superadmin_tenant_routes = Router::new()
        .route("/api/admin/all-tenants", get(tenant_handlers::list_all_tenants))
        .route("/api/admin/tenants/create", post(tenant_handlers::create_tenant))
        .route("/api/admin/all-users", get(tenant_handlers::list_all_users))
        .route("/api/admin/users/:user_id/assign-tenant", post(tenant_handlers::assign_user_to_tenant))
        .layer(axum_middleware::from_fn(middleware::auth_middleware));

    // 超管路由（超级管理员专用）
    let admin_routes = Router::new()
        // 租户管理
        .route("/api/admin/tenants", get(admin_handlers::list_tenants))
        .route("/api/admin/tenants", post(admin_handlers::create_tenant))
        .route("/api/admin/tenants/:tenant_id/status", patch(admin_handlers::update_tenant_status))
        .route("/api/admin/tenants/:tenant_id/users", get(admin_handlers::list_tenant_users))
        // 用户管理
        .route("/api/admin/users", get(admin_handlers::list_users))
        .route("/api/admin/tenant-users", post(admin_handlers::add_user_to_tenant))
        .route("/api/admin/tenant-users/:user_id/:tenant_id", delete(admin_handlers::remove_user_from_tenant))
        // 系统统计
        .route("/api/admin/stats", get(admin_handlers::get_system_stats))
        .layer(axum_middleware::from_fn(middleware::auth_middleware));

    // 数据 CRUD 路由（可选认证）
    let api_routes = Router::new()
        .route("/api/:schema/:table", get(handlers::get_records))
        .route("/api/:schema/:table", post(handlers::create_record))
        .route("/api/:schema/:table", patch(handlers::update_records))
        .route("/api/:schema/:table", delete(handlers::delete_records))
        .layer(axum_middleware::from_fn(middleware::optional_auth_middleware));

    // 合并所有路由
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(schema_routes)
        .merge(export_routes)
        .merge(monitor_routes)
        .merge(tenant_routes)
        .merge(superadmin_tenant_routes)
        .merge(admin_routes)
        .merge(api_routes)
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

/// 根路径处理器
async fn root_handler() -> Result<Json<Value>, AppError> {
    use serde_json::json;
    
    Ok(Json(json!({
        "name": "CrestRail API",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "endpoints": {
            "health": "/health",
            "admin": "/admin/",
            "api": "/api/:schema/:table",
            "auth": {
                "register": "/auth/register",
                "login": "/auth/login",
                "me": "/auth/me",
                "refresh": "/auth/refresh",
                "change_password": "/auth/change-password"
            },
            "transaction": "/transaction"
        },
        "documentation": "https://github.com/yourusername/crestrail"
    })))
}

/// SQL 查询执行端点
#[derive(serde::Deserialize)]
struct SqlQueryRequest {
    sql: String,
}

async fn execute_sql_query(
    State(pool): State<PgPool>,
    Json(req): Json<SqlQueryRequest>,
) -> Result<Json<Value>, AppError> {
    use serde_json::json;
    use sqlx::{Row, Column};
    
    let start = std::time::Instant::now();
    
    // 基本的 SQL 注入防护：只允许 SELECT 语句
    let sql_upper = req.sql.trim().to_uppercase();
    if !sql_upper.starts_with("SELECT") && !sql_upper.starts_with("WITH") {
        return Err(AppError::InvalidQuery(
            "只允许执行 SELECT 查询语句".to_string(),
        ));
    }
    
    // 执行查询
    let rows = sqlx::query(&req.sql)
        .fetch_all(&pool)
        .await?;
    
    // 转换为 JSON（使用更智能的类型处理）
    let results: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for column in row.columns() {
                let key = column.name().to_string();
                let idx = column.ordinal();
                
                // 尝试不同的类型
                let value: Value = if let Ok(v) = row.try_get::<String, _>(idx) {
                    Value::String(v)
                } else if let Ok(v) = row.try_get::<i32, _>(idx) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<i64, _>(idx) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<f64, _>(idx) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<bool, _>(idx) {
                    Value::Bool(v)
                } else if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
                    v.map(Value::String).unwrap_or(Value::Null)
                } else if let Ok(v) = row.try_get::<Option<i32>, _>(idx) {
                    v.map(|n| serde_json::json!(n)).unwrap_or(Value::Null)
                } else if let Ok(v) = row.try_get::<Option<i64>, _>(idx) {
                    v.map(|n| serde_json::json!(n)).unwrap_or(Value::Null)
                } else if let Ok(v) = row.try_get::<serde_json::Value, _>(idx) {
                    v
                } else {
                    Value::Null
                };
                
                obj.insert(key, value);
            }
            Value::Object(obj)
        })
        .collect();
    
    let elapsed = start.elapsed().as_millis();
    
    Ok(Json(json!({
        "data": results,
        "elapsed_ms": elapsed,
        "row_count": results.len()
    })))
}

/// 健康检查端点
async fn health_check(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    use serde_json::json;
    
    // 检查数据库连接
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    
    // 获取连接池状态
    let pool_size = pool.size();
    let idle_connections = pool.num_idle();
    
    Ok(Json(json!({
        "status": if db_status == "healthy" { "healthy" } else { "unhealthy" },
        "database": {
            "status": db_status,
            "connected": db_status == "healthy"
        },
        "pool": {
            "size": pool_size,
            "idle": idle_connections,
            "active": pool_size - (idle_connections as u32)
        },
        "version": env!("CARGO_PKG_VERSION")
    })))
}

