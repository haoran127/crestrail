use crate::error::Result;
use crate::query_builder::{QueryParams, SqlBuilder};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

/// GET /api/:schema/:table - 查询数据
pub async fn get_records(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>> {
    tracing::debug!("GET /api/{}/{} - 查询参数: {:?}", schema, table, query);

    // 解析查询参数
    let params = QueryParams::from_query_map(query)?;

    // 构建 SQL
    let builder = SqlBuilder::new(schema, table, params)?;
    let (sql, args) = builder.build_select()?;

    tracing::debug!("执行 SQL: {}", sql);

    // 执行查询
    let rows = sqlx::query_with(&sql, args)
        .fetch_all(&pool)
        .await?;

    // 转换为 JSON
    let results: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: Option<serde_json::Value> = row.try_get(i).ok();
                if let Some(v) = value {
                    obj.insert(column.name().to_string(), v);
                }
            }
            Value::Object(obj)
        })
        .collect();

    Ok(Json(Value::Array(results)))
}

/// POST /api/:schema/:table - 插入数据
pub async fn create_record(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Json(data): Json<Value>,
) -> Result<(StatusCode, Json<Value>)> {
    tracing::debug!("POST /api/{}/{} - 数据: {:?}", schema, table, data);

    // 处理批量插入或单条插入
    let records = if data.is_array() {
        data.as_array().unwrap().clone()
    } else {
        vec![data]
    };

    let mut results = Vec::new();

    for record in records {
        // 构建 SQL
        let builder = SqlBuilder::new(schema.clone(), table.clone(), QueryParams::default())?;
        let (sql, args) = builder.build_insert(&record)?;

        tracing::debug!("执行 SQL: {}", sql);

        // 执行插入
        let row = sqlx::query_with(&sql, args)
            .fetch_one(&pool)
            .await?;

        // 转换为 JSON
        let mut obj = serde_json::Map::new();
        for (i, column) in row.columns().iter().enumerate() {
            let value: Option<serde_json::Value> = row.try_get(i).ok();
            if let Some(v) = value {
                obj.insert(column.name().to_string(), v);
            }
        }
        results.push(Value::Object(obj));
    }

    let response = if results.len() == 1 {
        results.into_iter().next().unwrap()
    } else {
        Value::Array(results)
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// PATCH /api/:schema/:table - 更新数据
pub async fn update_records(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
    Json(data): Json<Value>,
) -> Result<Json<Value>> {
    tracing::debug!(
        "PATCH /api/{}/{} - 查询参数: {:?}, 数据: {:?}",
        schema,
        table,
        query,
        data
    );

    // 解析查询参数
    let params = QueryParams::from_query_map(query)?;

    // 构建 SQL
    let builder = SqlBuilder::new(schema, table, params)?;
    let (sql, args) = builder.build_update(&data)?;

    tracing::debug!("执行 SQL: {}", sql);

    // 执行更新
    let rows = sqlx::query_with(&sql, args)
        .fetch_all(&pool)
        .await?;

    // 转换为 JSON
    let results: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: Option<serde_json::Value> = row.try_get(i).ok();
                if let Some(v) = value {
                    obj.insert(column.name().to_string(), v);
                }
            }
            Value::Object(obj)
        })
        .collect();

    Ok(Json(Value::Array(results)))
}

/// DELETE /api/:schema/:table - 删除数据
pub async fn delete_records(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<(StatusCode, Json<Value>)> {
    tracing::debug!("DELETE /api/{}/{} - 查询参数: {:?}", schema, table, query);

    // 解析查询参数
    let params = QueryParams::from_query_map(query)?;

    // 构建 SQL
    let builder = SqlBuilder::new(schema, table, params)?;
    let (sql, args) = builder.build_delete()?;

    tracing::debug!("执行 SQL: {}", sql);

    // 执行删除
    let rows = sqlx::query_with(&sql, args)
        .fetch_all(&pool)
        .await?;

    // 转换为 JSON
    let results: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: Option<serde_json::Value> = row.try_get(i).ok();
                if let Some(v) = value {
                    obj.insert(column.name().to_string(), v);
                }
            }
            Value::Object(obj)
        })
        .collect();

    Ok((StatusCode::OK, Json(Value::Array(results))))
}

