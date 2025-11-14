use crate::error::Result;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

/// Schema 信息
#[derive(Debug, Serialize)]
pub struct SchemaInfo {
    pub schema_name: String,
    pub table_count: i64,
}

/// 表信息
#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub table_name: String,
    pub table_type: String,
    pub row_count: Option<i64>,
    pub size: Option<String>,
}

/// 列信息
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: String,
    pub column_default: Option<String>,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub ordinal_position: i32,
}

/// 约束信息
#[derive(Debug, Serialize)]
pub struct ConstraintInfo {
    pub constraint_name: String,
    pub constraint_type: String,
    pub column_name: Option<String>,
    pub foreign_table: Option<String>,
    pub foreign_column: Option<String>,
}

/// 索引信息
#[derive(Debug, Serialize)]
pub struct IndexInfo {
    pub index_name: String,
    pub index_type: String,
    pub is_unique: bool,
    pub is_primary: bool,
    pub columns: Vec<String>,
    pub index_def: String,
}

/// 外键信息
#[derive(Debug, Serialize)]
pub struct ForeignKeyInfo {
    pub constraint_name: String,
    pub column_name: String,
    pub referenced_table: String,
    pub referenced_column: String,
}

/// 表结构详情
#[derive(Debug, Serialize)]
pub struct TableStructure {
    pub schema_name: String,
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub constraints: Vec<ConstraintInfo>,
    pub indexes: Vec<IndexInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub row_count: Option<i64>,
    pub table_size: Option<String>,
}

/// GET /api/schemas - 获取所有 schema 列表
pub async fn list_schemas(State(pool): State<PgPool>) -> Result<Json<Vec<SchemaInfo>>> {
    let schemas = sqlx::query(
        r#"
        SELECT 
            table_schema as schema_name,
            COUNT(table_name) as table_count
        FROM information_schema.tables
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        GROUP BY table_schema
        ORDER BY table_schema
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let result: Vec<SchemaInfo> = schemas
        .iter()
        .map(|row| SchemaInfo {
            schema_name: row.get("schema_name"),
            table_count: row.get("table_count"),
        })
        .collect();

    Ok(Json(result))
}

/// GET /api/schema/:schema/tables - 获取指定 schema 的所有表
pub async fn list_tables(
    State(pool): State<PgPool>,
    Path(schema): Path<String>,
) -> Result<Json<Vec<TableInfo>>> {
    let tables = sqlx::query(
        r#"
        SELECT 
            t.table_name,
            t.table_type,
            pg_class.reltuples::bigint as row_count,
            pg_size_pretty(pg_total_relation_size(quote_ident(t.table_schema)||'.'||quote_ident(t.table_name))) as size
        FROM information_schema.tables t
        LEFT JOIN pg_class ON pg_class.relname = t.table_name
        LEFT JOIN pg_namespace ON pg_namespace.nspname = t.table_schema 
            AND pg_class.relnamespace = pg_namespace.oid
        WHERE t.table_schema = $1
        ORDER BY t.table_name
        "#,
    )
    .bind(&schema)
    .fetch_all(&pool)
    .await?;

    let result: Vec<TableInfo> = tables
        .iter()
        .map(|row| TableInfo {
            table_name: row.get("table_name"),
            table_type: row.get("table_type"),
            row_count: row.try_get("row_count").ok(),
            size: row.try_get("size").ok(),
        })
        .collect();

    Ok(Json(result))
}

/// GET /api/schema/:schema/table/:table/structure - 获取表结构详情
pub async fn get_table_structure(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
) -> Result<Json<TableStructure>> {
    // 获取列信息
    let columns = sqlx::query(
        r#"
        SELECT 
            column_name,
            data_type,
            is_nullable,
            column_default,
            character_maximum_length,
            numeric_precision,
            numeric_scale,
            ordinal_position
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position
        "#,
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(&pool)
    .await?;

    let columns_info: Vec<ColumnInfo> = columns
        .iter()
        .map(|row| ColumnInfo {
            column_name: row.get("column_name"),
            data_type: row.get("data_type"),
            is_nullable: row.get("is_nullable"),
            column_default: row.try_get("column_default").ok(),
            character_maximum_length: row.try_get("character_maximum_length").ok(),
            numeric_precision: row.try_get("numeric_precision").ok(),
            numeric_scale: row.try_get("numeric_scale").ok(),
            ordinal_position: row.get("ordinal_position"),
        })
        .collect();

    // 获取约束信息
    let constraints = sqlx::query(
        r#"
        SELECT 
            tc.constraint_name,
            tc.constraint_type,
            kcu.column_name,
            ccu.table_name AS foreign_table,
            ccu.column_name AS foreign_column
        FROM information_schema.table_constraints tc
        LEFT JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        LEFT JOIN information_schema.constraint_column_usage ccu
            ON tc.constraint_name = ccu.constraint_name
            AND tc.table_schema = ccu.table_schema
        WHERE tc.table_schema = $1 AND tc.table_name = $2
        ORDER BY tc.constraint_type, tc.constraint_name
        "#,
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(&pool)
    .await?;

    let constraints_info: Vec<ConstraintInfo> = constraints
        .iter()
        .map(|row| ConstraintInfo {
            constraint_name: row.get("constraint_name"),
            constraint_type: row.get("constraint_type"),
            column_name: row.try_get("column_name").ok(),
            foreign_table: row.try_get("foreign_table").ok(),
            foreign_column: row.try_get("foreign_column").ok(),
        })
        .collect();

    // 获取索引信息
    let indexes = sqlx::query(
        r#"
        SELECT
            i.relname as index_name,
            am.amname as index_type,
            ix.indisunique as is_unique,
            ix.indisprimary as is_primary,
            array_agg(a.attname ORDER BY a.attnum) as columns,
            pg_get_indexdef(ix.indexrelid) as index_def
        FROM pg_class t
        JOIN pg_namespace n ON n.oid = t.relnamespace
        JOIN pg_index ix ON t.oid = ix.indrelid
        JOIN pg_class i ON i.oid = ix.indexrelid
        JOIN pg_am am ON i.relam = am.oid
        JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
        WHERE n.nspname = $1 AND t.relname = $2
        GROUP BY i.relname, am.amname, ix.indisunique, ix.indisprimary, ix.indexrelid
        ORDER BY i.relname
        "#,
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(&pool)
    .await?;

    let indexes_info: Vec<IndexInfo> = indexes
        .iter()
        .map(|row| {
            let columns_array: Vec<String> = row.get::<Vec<String>, _>("columns");
            IndexInfo {
                index_name: row.get("index_name"),
                index_type: row.get("index_type"),
                is_unique: row.get("is_unique"),
                is_primary: row.get("is_primary"),
                columns: columns_array,
                index_def: row.get("index_def"),
            }
        })
        .collect();

    // 获取行数和表大小
    let stats = sqlx::query(
        r#"
        SELECT 
            pg_class.reltuples::bigint as row_count,
            pg_size_pretty(pg_total_relation_size($1||'.'||$2)) as table_size
        FROM pg_class
        JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace
        WHERE pg_namespace.nspname = $1 AND pg_class.relname = $2
        "#,
    )
    .bind(&schema)
    .bind(&table)
    .fetch_optional(&pool)
    .await?;

    let (row_count, table_size) = if let Some(row) = stats {
        (
            row.try_get("row_count").ok(),
            row.try_get("table_size").ok(),
        )
    } else {
        (None, None)
    };

    // 从 constraints 中提取外键信息
    let foreign_keys_info: Vec<ForeignKeyInfo> = constraints_info
        .iter()
        .filter(|c| c.constraint_type == "FOREIGN KEY")
        .filter_map(|c| {
            if let (Some(col), Some(ftable), Some(fcol)) = 
                (&c.column_name, &c.foreign_table, &c.foreign_column) {
                Some(ForeignKeyInfo {
                    constraint_name: c.constraint_name.clone(),
                    column_name: col.clone(),
                    referenced_table: ftable.clone(),
                    referenced_column: fcol.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(Json(TableStructure {
        schema_name: schema,
        table_name: table,
        columns: columns_info,
        constraints: constraints_info,
        indexes: indexes_info,
        foreign_keys: foreign_keys_info,
        row_count,
        table_size,
    }))
}

/// GET /api/schema/:schema/table/:table/relationships - 获取表的关系图数据
pub async fn get_table_relationships(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
) -> Result<Json<Value>> {
    // 获取该表引用的其他表（外键）
    let foreign_keys = sqlx::query(
        r#"
        SELECT
            tc.constraint_name,
            kcu.column_name,
            ccu.table_schema AS foreign_schema,
            ccu.table_name AS foreign_table,
            ccu.column_name AS foreign_column
        FROM information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        JOIN information_schema.constraint_column_usage AS ccu
            ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = $1
            AND tc.table_name = $2
        "#,
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(&pool)
    .await?;

    // 获取引用该表的其他表（被引用）
    let referenced_by = sqlx::query(
        r#"
        SELECT
            tc.table_schema,
            tc.table_name,
            tc.constraint_name,
            kcu.column_name,
            ccu.column_name AS foreign_column
        FROM information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        JOIN information_schema.constraint_column_usage AS ccu
            ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND ccu.table_schema = $1
            AND ccu.table_name = $2
        "#,
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(&pool)
    .await?;

    let fk_list: Vec<Value> = foreign_keys
        .iter()
        .map(|row| {
            serde_json::json!({
                "constraint_name": row.get::<String, _>("constraint_name"),
                "column": row.get::<String, _>("column_name"),
                "foreign_schema": row.get::<String, _>("foreign_schema"),
                "foreign_table": row.get::<String, _>("foreign_table"),
                "foreign_column": row.get::<String, _>("foreign_column"),
            })
        })
        .collect();

    let ref_list: Vec<Value> = referenced_by
        .iter()
        .map(|row| {
            serde_json::json!({
                "schema": row.get::<String, _>("table_schema"),
                "table": row.get::<String, _>("table_name"),
                "constraint_name": row.get::<String, _>("constraint_name"),
                "column": row.get::<String, _>("column_name"),
                "foreign_column": row.get::<String, _>("foreign_column"),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "foreign_keys": fk_list,
        "referenced_by": ref_list,
    })))
}

