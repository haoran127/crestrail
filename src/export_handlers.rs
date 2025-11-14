use crate::error::Result;
use crate::query_builder::{QueryParams, SqlBuilder};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use sqlx::{Column, PgPool, Row};
use std::collections::HashMap;

/// GET /api/export/csv/:schema/:table - 导出为 CSV
pub async fn export_csv(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response> {
    // 解析查询参数
    let params = QueryParams::from_query_map(query)?;

    // 构建 SQL
    let builder = SqlBuilder::new(schema.clone(), table.clone(), params)?;
    let (sql, args) = builder.build_select()?;

    tracing::debug!("导出 CSV - 执行 SQL: {}", sql);

    // 执行查询
    let rows = sqlx::query_with(&sql, args).fetch_all(&pool).await?;

    if rows.is_empty() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/csv"),
        );
        return Ok((StatusCode::OK, headers, "".to_string()).into_response());
    }

    // 构建 CSV
    let mut csv = String::new();

    // CSV 头部（列名）
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|col| escape_csv_field(col.name()))
        .collect();
    csv.push_str(&columns.join(","));
    csv.push('\n');

    // CSV 数据行
    for row in &rows {
        let mut values = Vec::new();
        for (i, _column) in row.columns().iter().enumerate() {
            let value = get_value_as_string(&row, i);
            values.push(escape_csv_field(&value));
        }
        csv.push_str(&values.join(","));
        csv.push('\n');
    }

    // 设置响应头
    let filename = format!("{}_{}.csv", schema, table);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
    );

    Ok((StatusCode::OK, headers, csv).into_response())
}

/// GET /api/export/json/:schema/:table - 导出为 JSON
pub async fn export_json(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response> {
    // 解析查询参数
    let params = QueryParams::from_query_map(query)?;

    // 构建 SQL
    let builder = SqlBuilder::new(schema.clone(), table.clone(), params)?;
    let (sql, args) = builder.build_select()?;

    tracing::debug!("导出 JSON - 执行 SQL: {}", sql);

    // 执行查询
    let rows = sqlx::query_with(&sql, args).fetch_all(&pool).await?;

    // 转换为 JSON
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

    let json_str = serde_json::to_string_pretty(&results)?;

    // 设置响应头
    let filename = format!("{}_{}.json", schema, table);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
    );

    Ok((StatusCode::OK, headers, json_str).into_response())
}

/// POST /api/export/sql - 导出 SQL 查询结果为 CSV
pub async fn export_sql_csv(
    State(pool): State<PgPool>,
    Json(req): Json<serde_json::Value>,
) -> Result<Response> {
    let sql = req
        .get("sql")
        .and_then(|s| s.as_str())
        .ok_or_else(|| crate::error::AppError::InvalidQuery("缺少 sql 参数".to_string()))?;

    // 基本的 SQL 注入防护：只允许 SELECT 语句
    let sql_upper = sql.trim().to_uppercase();
    if !sql_upper.starts_with("SELECT") && !sql_upper.starts_with("WITH") {
        return Err(crate::error::AppError::InvalidQuery(
            "只允许执行 SELECT 查询语句".to_string(),
        ));
    }

    tracing::debug!("导出 SQL 查询结果 - 执行 SQL: {}", sql);

    // 执行查询
    let rows = sqlx::query(sql).fetch_all(&pool).await?;

    if rows.is_empty() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/csv"),
        );
        return Ok((StatusCode::OK, headers, "".to_string()).into_response());
    }

    // 构建 CSV
    let mut csv = String::new();

    // CSV 头部（列名）
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|col| escape_csv_field(col.name()))
        .collect();
    csv.push_str(&columns.join(","));
    csv.push('\n');

    // CSV 数据行
    for row in &rows {
        let mut values = Vec::new();
        for (i, _column) in row.columns().iter().enumerate() {
            let value = get_value_as_string(&row, i);
            values.push(escape_csv_field(&value));
        }
        csv.push_str(&values.join(","));
        csv.push('\n');
    }

    // 设置响应头
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"query_result.csv\""),
    );

    Ok((StatusCode::OK, headers, csv).into_response())
}

/// 转义 CSV 字段
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// 从行中获取值作为字符串
fn get_value_as_string(row: &sqlx::postgres::PgRow, idx: usize) -> String {
    if let Ok(v) = row.try_get::<String, _>(idx) {
        v
    } else if let Ok(v) = row.try_get::<i32, _>(idx) {
        v.to_string()
    } else if let Ok(v) = row.try_get::<i64, _>(idx) {
        v.to_string()
    } else if let Ok(v) = row.try_get::<f64, _>(idx) {
        v.to_string()
    } else if let Ok(v) = row.try_get::<bool, _>(idx) {
        v.to_string()
    } else if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
        v.unwrap_or_else(|| "NULL".to_string())
    } else if let Ok(v) = row.try_get::<Option<i32>, _>(idx) {
        v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".to_string())
    } else if let Ok(v) = row.try_get::<Option<i64>, _>(idx) {
        v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".to_string())
    } else if let Ok(v) = row.try_get::<serde_json::Value, _>(idx) {
        v.to_string()
    } else {
        "NULL".to_string()
    }
}

