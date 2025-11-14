use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Column, PgPool, Postgres, Row, Transaction};
use std::collections::HashMap;

use crate::error::AppError;
use crate::query_builder::QueryParams;

/// 事务操作类型
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum OperationType {
    Post,   // 插入
    Patch,  // 更新
    Delete, // 删除
}

/// 单个事务操作
#[derive(Debug, Deserialize)]
pub struct TransactionOperation {
    /// 操作类型
    pub method: OperationType,
    /// Schema 名称
    pub schema: String,
    /// 表名
    pub table: String,
    /// WHERE 条件（用于 PATCH 和 DELETE）
    #[serde(rename = "where")]
    pub conditions: Option<HashMap<String, String>>,
    /// 数据（用于 POST 和 PATCH）
    pub data: Option<Value>,
}

/// 事务请求
#[derive(Debug, Deserialize)]
pub struct TransactionRequest {
    /// 操作列表
    pub operations: Vec<TransactionOperation>,
}

/// 事务响应
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    /// 成功的操作数
    pub success_count: usize,
    /// 每个操作的结果
    pub results: Vec<Value>,
    /// 总耗时（毫秒）
    pub elapsed_ms: u128,
}

/// 执行事务
pub async fn execute_transaction(
    State(pool): State<PgPool>,
    Json(req): Json<TransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
    let start = std::time::Instant::now();

    // 验证操作不为空
    if req.operations.is_empty() {
        return Err(AppError::InvalidQuery("事务操作列表不能为空".to_string()));
    }

    // 验证操作数量限制（防止超大事务）
    const MAX_OPERATIONS: usize = 100;
    if req.operations.len() > MAX_OPERATIONS {
        return Err(AppError::InvalidQuery(format!(
            "单个事务最多支持 {} 个操作",
            MAX_OPERATIONS
        )));
    }

    // 开启事务
    let mut tx = pool.begin().await?;

    let mut results = Vec::new();

    // 执行每个操作
    for (index, op) in req.operations.iter().enumerate() {
        tracing::debug!(
            "执行事务操作 {}/{}: {:?} {}.{}",
            index + 1,
            req.operations.len(),
            op.method,
            op.schema,
            op.table
        );

        let result = match op.method {
            OperationType::Post => execute_insert(&mut tx, op).await?,
            OperationType::Patch => execute_update(&mut tx, op).await?,
            OperationType::Delete => execute_delete(&mut tx, op).await?,
        };

        results.push(result);
    }

    // 提交事务
    tx.commit().await.map_err(|e| {
        tracing::error!("事务提交失败: {}", e);
        AppError::Database(e)
    })?;

    let elapsed = start.elapsed().as_millis();

    tracing::info!(
        "事务执行成功: {} 个操作，耗时 {}ms",
        results.len(),
        elapsed
    );

    Ok((
        StatusCode::OK,
        Json(TransactionResponse {
            success_count: results.len(),
            results,
            elapsed_ms: elapsed,
        }),
    ))
}

/// 执行插入操作
async fn execute_insert(
    tx: &mut Transaction<'_, Postgres>,
    op: &TransactionOperation,
) -> Result<Value, AppError> {
    let data = op
        .data
        .as_ref()
        .ok_or_else(|| AppError::InvalidQuery("POST 操作需要提供 data 字段".to_string()))?;

    // 验证标识符
    QueryParams::sanitize_identifier(&op.schema)?;
    QueryParams::sanitize_identifier(&op.table)?;

    // 构建 INSERT SQL
    let obj = data
        .as_object()
        .ok_or_else(|| AppError::InvalidQuery("data 必须是 JSON 对象".to_string()))?;

    if obj.is_empty() {
        return Err(AppError::InvalidQuery("data 不能为空".to_string()));
    }

    let mut columns = Vec::new();
    let mut placeholders = Vec::new();

    for (i, key) in obj.keys().enumerate() {
        QueryParams::sanitize_identifier(key)?;
        columns.push(format!("\"{}\"", key));
        placeholders.push(format!("${}", i + 1));
    }

    let sql = format!(
        "INSERT INTO \"{}\".\"{}\" ({}) VALUES ({}) RETURNING *",
        op.schema,
        op.table,
        columns.join(", "),
        placeholders.join(", ")
    );

    tracing::debug!("执行 SQL: {}", sql);

    // 构建查询
    let mut query = sqlx::query(&sql);
    for value in obj.values() {
        query = bind_json_value(query, value);
    }

    // 执行并返回结果
    let rows = query.fetch_all(&mut **tx).await?;
    rows_to_json(rows)
}

/// 执行更新操作
async fn execute_update(
    tx: &mut Transaction<'_, Postgres>,
    op: &TransactionOperation,
) -> Result<Value, AppError> {
    let data = op
        .data
        .as_ref()
        .ok_or_else(|| AppError::InvalidQuery("PATCH 操作需要提供 data 字段".to_string()))?;

    let conditions = op.conditions.as_ref().ok_or_else(|| {
        AppError::InvalidQuery("PATCH 操作需要提供 where 条件".to_string())
    })?;

    // 验证标识符
    QueryParams::sanitize_identifier(&op.schema)?;
    QueryParams::sanitize_identifier(&op.table)?;

    // 构建 UPDATE SQL
    let obj = data
        .as_object()
        .ok_or_else(|| AppError::InvalidQuery("data 必须是 JSON 对象".to_string()))?;

    if obj.is_empty() {
        return Err(AppError::InvalidQuery("data 不能为空".to_string()));
    }

    let mut set_clauses = Vec::new();
    let mut param_index = 1;

    for key in obj.keys() {
        QueryParams::sanitize_identifier(key)?;
        set_clauses.push(format!("\"{}\" = ${}", key, param_index));
        param_index += 1;
    }

    let mut sql = format!(
        "UPDATE \"{}\".\"{}\" SET {}",
        op.schema,
        op.table,
        set_clauses.join(", ")
    );

    // 添加 WHERE 条件
    let mut where_clauses = Vec::new();
    for (key, _) in conditions.iter() {
        QueryParams::sanitize_identifier(key)?;
        where_clauses.push(format!("\"{}\" = ${}", key, param_index));
        param_index += 1;
    }

    if !where_clauses.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_clauses.join(" AND ")));
    }

    sql.push_str(" RETURNING *");

    tracing::debug!("执行 SQL: {}", sql);

    // 构建查询
    let mut query = sqlx::query(&sql);
    
    // 绑定 SET 值
    for value in obj.values() {
        query = bind_json_value(query, value);
    }
    
    // 绑定 WHERE 值
    for value in conditions.values() {
        query = query.bind(value);
    }

    let rows = query.fetch_all(&mut **tx).await?;
    rows_to_json(rows)
}

/// 执行删除操作
async fn execute_delete(
    tx: &mut Transaction<'_, Postgres>,
    op: &TransactionOperation,
) -> Result<Value, AppError> {
    let conditions = op.conditions.as_ref().ok_or_else(|| {
        AppError::InvalidQuery("DELETE 操作需要提供 where 条件".to_string())
    })?;

    // 验证标识符
    QueryParams::sanitize_identifier(&op.schema)?;
    QueryParams::sanitize_identifier(&op.table)?;

    // 构建 DELETE SQL
    let mut sql = format!("DELETE FROM \"{}\".\"{}\"", op.schema, op.table);

    let mut where_clauses = Vec::new();
    let mut param_index = 1;

    for (key, _) in conditions.iter() {
        QueryParams::sanitize_identifier(key)?;
        where_clauses.push(format!("\"{}\" = ${}", key, param_index));
        param_index += 1;
    }

    if where_clauses.is_empty() {
        return Err(AppError::InvalidQuery(
            "DELETE 操作必须提供 WHERE 条件，以防止误删除全表".to_string(),
        ));
    }

    sql.push_str(&format!(" WHERE {}", where_clauses.join(" AND ")));
    sql.push_str(" RETURNING *");

    tracing::debug!("执行 SQL: {}", sql);

    // 构建查询
    let mut query = sqlx::query(&sql);
    for value in conditions.values() {
        query = query.bind(value);
    }

    let rows = query.fetch_all(&mut **tx).await?;
    rows_to_json(rows)
}

/// 绑定 JSON 值到 SQL 查询
fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    value: &'q Value,
) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(b) => query.bind(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        Value::String(s) => query.bind(s.as_str()),
        Value::Array(_) | Value::Object(_) => query.bind(value.to_string()),
    }
}

/// 将数据库行转换为 JSON
fn rows_to_json(rows: Vec<sqlx::postgres::PgRow>) -> Result<Value, AppError> {
    let mut result = Vec::new();

    for row in rows {
        let mut obj = serde_json::Map::new();

        for column in row.columns() {
            let key = column.name().to_string();
            let idx = column.ordinal();

            // 尝试获取不同类型的值
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

        result.push(Value::Object(obj));
    }

    Ok(Value::Array(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_request_deserialization() {
        let json = r#"{
            "operations": [
                {
                    "method": "POST",
                    "schema": "public",
                    "table": "users",
                    "data": {
                        "name": "Test User",
                        "email": "test@example.com"
                    }
                },
                {
                    "method": "PATCH",
                    "schema": "public",
                    "table": "users",
                    "where": {"id": "1"},
                    "data": {"status": "active"}
                }
            ]
        }"#;

        let req: TransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.operations.len(), 2);
    }
}
