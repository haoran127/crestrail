use crate::error::Result;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::{PgPool, Row};

/// 数据库统计信息
#[derive(Debug, Serialize)]
pub struct DatabaseStats {
    pub database_size: String,
    pub table_count: i64,
    pub connection_count: i32,
    pub max_connections: i32,
    pub active_connections: i64,
    pub idle_connections: i64,
    pub cache_hit_ratio: f64,
    pub transaction_count: i64,
    pub uptime_seconds: i64,
}

/// 表大小统计
#[derive(Debug, Serialize)]
pub struct TableSizeInfo {
    pub schema_name: String,
    pub table_name: String,
    pub row_count: i64,
    pub total_size: String,
    pub table_size: String,
    pub index_size: String,
}

/// 慢查询信息
#[derive(Debug, Serialize)]
pub struct SlowQuery {
    pub query: String,
    pub calls: i64,
    pub total_time: f64,
    pub mean_time: f64,
    pub max_time: f64,
}

/// GET /api/monitor/stats - 获取数据库统计信息
pub async fn get_database_stats(State(pool): State<PgPool>) -> Result<Json<DatabaseStats>> {
    // 数据库大小
    let db_size: String = sqlx::query_scalar(
        "SELECT pg_size_pretty(pg_database_size(current_database()))"
    )
    .fetch_one(&pool)
    .await?;

    // 表数量
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema')"
    )
    .fetch_one(&pool)
    .await?;

    // 连接数信息
    let conn_info = sqlx::query(
        r#"
        SELECT 
            (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_conn,
            COUNT(*) FILTER (WHERE state = 'active') as active,
            COUNT(*) FILTER (WHERE state = 'idle') as idle,
            COUNT(*) as total
        FROM pg_stat_activity
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let max_connections: i32 = conn_info.get("max_conn");
    let active_connections: i64 = conn_info.get("active");
    let idle_connections: i64 = conn_info.get("idle");
    let connection_count: i64 = conn_info.get("total");

    // 缓存命中率
    let cache_stats = sqlx::query(
        r#"
        SELECT 
            sum(blks_hit) / NULLIF(sum(blks_hit) + sum(blks_read), 0) as ratio
        FROM pg_stat_database
        WHERE datname = current_database()
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let cache_hit_ratio: Option<f64> = cache_stats.try_get("ratio").ok();

    // 事务数
    let tx_count: i64 = sqlx::query_scalar(
        "SELECT xact_commit + xact_rollback FROM pg_stat_database WHERE datname = current_database()"
    )
    .fetch_one(&pool)
    .await?;

    // 运行时间（秒）
    let uptime: f64 = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM (now() - pg_postmaster_start_time()))"
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(DatabaseStats {
        database_size: db_size,
        table_count,
        connection_count: connection_count as i32,
        max_connections,
        active_connections,
        idle_connections,
        cache_hit_ratio: cache_hit_ratio.unwrap_or(0.0) * 100.0,
        transaction_count: tx_count,
        uptime_seconds: uptime as i64,
    }))
}

/// GET /api/monitor/tables - 获取表大小统计（Top 10）
pub async fn get_table_sizes(State(pool): State<PgPool>) -> Result<Json<Vec<TableSizeInfo>>> {
    let tables = sqlx::query(
        r#"
        SELECT
            schemaname as schema_name,
            tablename as table_name,
            pg_class.reltuples::bigint as row_count,
            pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
            pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
            pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) as index_size
        FROM pg_tables
        LEFT JOIN pg_class ON pg_class.relname = tablename
        LEFT JOIN pg_namespace ON pg_namespace.nspname = schemaname AND pg_class.relnamespace = pg_namespace.oid
        WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
        ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let result: Vec<TableSizeInfo> = tables
        .iter()
        .map(|row| TableSizeInfo {
            schema_name: row.get("schema_name"),
            table_name: row.get("table_name"),
            row_count: row.try_get("row_count").unwrap_or(0),
            total_size: row.get("total_size"),
            table_size: row.get("table_size"),
            index_size: row.get("index_size"),
        })
        .collect();

    Ok(Json(result))
}

/// GET /api/monitor/slow-queries - 获取慢查询（需要 pg_stat_statements 扩展）
pub async fn get_slow_queries(State(pool): State<PgPool>) -> Result<Json<Vec<SlowQuery>>> {
    // 检查 pg_stat_statements 是否已启用
    let extension_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'pg_stat_statements')"
    )
    .fetch_one(&pool)
    .await?;

    if !extension_exists {
        return Ok(Json(vec![]));
    }

    let queries = sqlx::query(
        r#"
        SELECT
            query,
            calls,
            total_exec_time as total_time,
            mean_exec_time as mean_time,
            max_exec_time as max_time
        FROM pg_stat_statements
        WHERE query NOT LIKE '%pg_stat_statements%'
        ORDER BY mean_exec_time DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&pool)
    .await;

    match queries {
        Ok(rows) => {
            let result: Vec<SlowQuery> = rows
                .iter()
                .map(|row| SlowQuery {
                    query: row.get("query"),
                    calls: row.get("calls"),
                    total_time: row.get("total_time"),
                    mean_time: row.get("mean_time"),
                    max_time: row.get("max_time"),
                })
                .collect();
            Ok(Json(result))
        }
        Err(_) => {
            // 如果查询失败（可能是权限问题），返回空列表
            Ok(Json(vec![]))
        }
    }
}

/// 活动连接信息
#[derive(Debug, Serialize)]
pub struct ActiveConnection {
    pub pid: i32,
    pub user: String,
    pub database: String,
    pub client_addr: Option<String>,
    pub state: String,
    pub query: String,
    pub duration_seconds: Option<f64>,
}

/// GET /api/monitor/connections - 获取活动连接
pub async fn get_active_connections(State(pool): State<PgPool>) -> Result<Json<Vec<ActiveConnection>>> {
    let connections = sqlx::query(
        r#"
        SELECT
            pid,
            usename as user,
            datname as database,
            client_addr::text,
            state,
            query,
            EXTRACT(EPOCH FROM (now() - query_start)) as duration
        FROM pg_stat_activity
        WHERE pid <> pg_backend_pid()
        AND state <> 'idle'
        ORDER BY query_start DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let result: Vec<ActiveConnection> = connections
        .iter()
        .map(|row| ActiveConnection {
            pid: row.get("pid"),
            user: row.get("user"),
            database: row.get("database"),
            client_addr: row.try_get("client_addr").ok(),
            state: row.get("state"),
            query: row.get("query"),
            duration_seconds: row.try_get("duration").ok(),
        })
        .collect();

    Ok(Json(result))
}

