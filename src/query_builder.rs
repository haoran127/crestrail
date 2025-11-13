use crate::error::{AppError, Result};
use sqlx::postgres::PgArguments;
use sqlx::Arguments;
use std::collections::HashMap;

/// 查询参数解析器
#[derive(Debug, Default)]
pub struct QueryParams {
    /// WHERE 条件
    pub filters: Vec<Filter>,
    /// 排序字段
    pub order_by: Vec<OrderBy>,
    /// 分页限制
    pub limit: Option<i64>,
    /// 偏移量
    pub offset: Option<i64>,
    /// 选择的字段
    pub select: Option<Vec<String>>,
}

/// 过滤条件
#[derive(Debug, Clone)]
pub struct Filter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: String,
}

/// 过滤操作符
#[derive(Debug, Clone)]
pub enum FilterOperator {
    Eq,          // 等于 (=)
    Neq,         // 不等于 (!=)
    Gt,          // 大于 (>)
    Gte,         // 大于等于 (>=)
    Lt,          // 小于 (<)
    Lte,         // 小于等于 (<=)
    Like,        // 模糊匹配 (LIKE)
    Ilike,       // 不区分大小写模糊匹配 (ILIKE)
    In,          // IN 查询
    Is,          // IS (用于 NULL)
}

/// 排序方向
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

impl QueryParams {
    /// 从 URL 查询参数解析
    pub fn from_query_map(query: HashMap<String, String>) -> Result<Self> {
        let mut params = QueryParams::default();

        for (key, value) in query.iter() {
            match key.as_str() {
                "limit" => {
                    params.limit = Some(
                        value
                            .parse()
                            .map_err(|_| AppError::InvalidQuery("limit 必须是数字".to_string()))?,
                    );
                }
                "offset" => {
                    params.offset = Some(
                        value
                            .parse()
                            .map_err(|_| AppError::InvalidQuery("offset 必须是数字".to_string()))?,
                    );
                }
                "order" => {
                    params.order_by = Self::parse_order(value)?;
                }
                "select" => {
                    params.select = Some(
                        value
                            .split(',')
                            .map(|s| Self::sanitize_identifier(s.trim()))
                            .collect::<Result<Vec<_>>>()?,
                    );
                }
                _ => {
                    // 处理过滤条件
                    if let Some(filter) = Self::parse_filter(key, value)? {
                        params.filters.push(filter);
                    }
                }
            }
        }

        Ok(params)
    }

    /// 解析过滤条件
    fn parse_filter(key: &str, value: &str) -> Result<Option<Filter>> {
        // 支持的格式:
        // column=value (等于)
        // column.eq=value (等于)
        // column.neq=value (不等于)
        // column.gt=value (大于)
        // column.gte=value (大于等于)
        // column.lt=value (小于)
        // column.lte=value (小于等于)
        // column.like=value (模糊匹配)
        // column.ilike=value (不区分大小写模糊匹配)
        // column.in=value1,value2,value3 (IN 查询)
        // column.is=null (IS NULL)

        let parts: Vec<&str> = key.split('.').collect();

        let (column, operator) = match parts.len() {
            1 => (parts[0], FilterOperator::Eq),
            2 => {
                let op = match parts[1] {
                    "eq" => FilterOperator::Eq,
                    "neq" => FilterOperator::Neq,
                    "gt" => FilterOperator::Gt,
                    "gte" => FilterOperator::Gte,
                    "lt" => FilterOperator::Lt,
                    "lte" => FilterOperator::Lte,
                    "like" => FilterOperator::Like,
                    "ilike" => FilterOperator::Ilike,
                    "in" => FilterOperator::In,
                    "is" => FilterOperator::Is,
                    _ => return Ok(None), // 忽略不支持的操作符
                };
                (parts[0], op)
            }
            _ => return Ok(None),
        };

        Self::sanitize_identifier(column)?;

        Ok(Some(Filter {
            column: column.to_string(),
            operator,
            value: value.to_string(),
        }))
    }

    /// 解析排序
    fn parse_order(order_str: &str) -> Result<Vec<OrderBy>> {
        let mut orders = Vec::new();

        for part in order_str.split(',') {
            let part = part.trim();
            let (column, ascending) = if let Some(col) = part.strip_suffix(".desc") {
                (col, false)
            } else if let Some(col) = part.strip_suffix(".asc") {
                (col, true)
            } else {
                (part, true) // 默认升序
            };

            Self::sanitize_identifier(column)?;
            orders.push(OrderBy {
                column: column.to_string(),
                ascending,
            });
        }

        Ok(orders)
    }

    /// 验证标识符安全性 (防止 SQL 注入)
    fn sanitize_identifier(ident: &str) -> Result<String> {
        // 只允许字母、数字、下划线
        if ident.is_empty() {
            return Err(AppError::InvalidQuery("标识符不能为空".to_string()));
        }

        if !ident
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(AppError::InvalidQuery(format!(
                "无效的标识符: {}. 只允许字母、数字和下划线",
                ident
            )));
        }

        // 不允许以数字开头
        if ident.chars().next().unwrap().is_ascii_digit() {
            return Err(AppError::InvalidQuery(
                "标识符不能以数字开头".to_string(),
            ));
        }

        Ok(ident.to_string())
    }
}

/// SQL 查询构建器
pub struct SqlBuilder {
    schema: String,
    table: String,
    params: QueryParams,
}

impl SqlBuilder {
    pub fn new(schema: String, table: String, params: QueryParams) -> Result<Self> {
        // 验证 schema 和 table 的安全性
        QueryParams::sanitize_identifier(&schema)?;
        QueryParams::sanitize_identifier(&table)?;

        Ok(Self {
            schema,
            table,
            params,
        })
    }

    /// 构建 SELECT 查询
    pub fn build_select(&self) -> Result<(String, PgArguments)> {
        let mut args = PgArguments::default();
        let mut arg_index = 1;

        // SELECT 字段
        let select_clause = match &self.params.select {
            Some(fields) => fields.join(", "),
            None => "*".to_string(),
        };

        let mut sql = format!(
            "SELECT {} FROM \"{}\".\"{}\"",
            select_clause, self.schema, self.table
        );

        // WHERE 条件
        if !self.params.filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .params
                .filters
                .iter()
                .map(|filter| {
                    let condition = match filter.operator {
                        FilterOperator::Eq => {
                            args.add(&filter.value);
                            format!("\"{}\" = ${}", filter.column, arg_index)
                        }
                        FilterOperator::Neq => {
                            args.add(&filter.value);
                            format!("\"{}\" != ${}", filter.column, arg_index)
                        }
                        FilterOperator::Gt => {
                            args.add(&filter.value);
                            format!("\"{}\" > ${}", filter.column, arg_index)
                        }
                        FilterOperator::Gte => {
                            args.add(&filter.value);
                            format!("\"{}\" >= ${}", filter.column, arg_index)
                        }
                        FilterOperator::Lt => {
                            args.add(&filter.value);
                            format!("\"{}\" < ${}", filter.column, arg_index)
                        }
                        FilterOperator::Lte => {
                            args.add(&filter.value);
                            format!("\"{}\" <= ${}", filter.column, arg_index)
                        }
                        FilterOperator::Like => {
                            args.add(&filter.value);
                            format!("\"{}\" LIKE ${}", filter.column, arg_index)
                        }
                        FilterOperator::Ilike => {
                            args.add(&filter.value);
                            format!("\"{}\" ILIKE ${}", filter.column, arg_index)
                        }
                        FilterOperator::In => {
                            let values: Vec<&str> = filter.value.split(',').collect();
                            let placeholders: Vec<String> = values
                                .iter()
                                .map(|v| {
                                    args.add(*v);
                                    let placeholder = format!("${}", arg_index);
                                    arg_index += 1;
                                    placeholder
                                })
                                .collect();
                            arg_index -= 1; // 调整因为循环会再加一次
                            format!("\"{}\" IN ({})", filter.column, placeholders.join(", "))
                        }
                        FilterOperator::Is => {
                            if filter.value.to_lowercase() == "null" {
                                format!("\"{}\" IS NULL", filter.column)
                            } else {
                                format!("\"{}\" IS NOT NULL", filter.column)
                            }
                        }
                    };
                    arg_index += 1;
                    condition
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // ORDER BY
        if !self.params.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self
                .params
                .order_by
                .iter()
                .map(|order| {
                    format!(
                        "\"{}\" {}",
                        order.column,
                        if order.ascending { "ASC" } else { "DESC" }
                    )
                })
                .collect();
            sql.push_str(&orders.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.params.limit {
            args.add(limit);
            sql.push_str(&format!(" LIMIT ${}", arg_index));
            arg_index += 1;
        }

        // OFFSET
        if let Some(offset) = self.params.offset {
            args.add(offset);
            sql.push_str(&format!(" OFFSET ${}", arg_index));
        }

        Ok((sql, args))
    }

    /// 构建 INSERT 查询
    pub fn build_insert(&self, data: &serde_json::Value) -> Result<(String, PgArguments)> {
        let mut args = PgArguments::default();

        let obj = data
            .as_object()
            .ok_or_else(|| AppError::InvalidJson(serde_json::Error::custom("期望 JSON 对象")))?;

        if obj.is_empty() {
            return Err(AppError::InvalidQuery("插入数据不能为空".to_string()));
        }

        let mut columns = Vec::new();
        let mut placeholders = Vec::new();

        for (i, (key, value)) in obj.iter().enumerate() {
            QueryParams::sanitize_identifier(key)?;
            columns.push(format!("\"{}\"", key));
            placeholders.push(format!("${}", i + 1));
            args.add(value);
        }

        let sql = format!(
            "INSERT INTO \"{}\".\"{}\" ({}) VALUES ({}) RETURNING *",
            self.schema,
            self.table,
            columns.join(", "),
            placeholders.join(", ")
        );

        Ok((sql, args))
    }

    /// 构建 UPDATE 查询
    pub fn build_update(&self, data: &serde_json::Value) -> Result<(String, PgArguments)> {
        let mut args = PgArguments::default();
        let mut arg_index = 1;

        let obj = data
            .as_object()
            .ok_or_else(|| AppError::InvalidJson(serde_json::Error::custom("期望 JSON 对象")))?;

        if obj.is_empty() {
            return Err(AppError::InvalidQuery("更新数据不能为空".to_string()));
        }

        let mut set_clauses = Vec::new();
        for (key, value) in obj.iter() {
            QueryParams::sanitize_identifier(key)?;
            set_clauses.push(format!("\"{}\" = ${}", key, arg_index));
            args.add(value);
            arg_index += 1;
        }

        let mut sql = format!(
            "UPDATE \"{}\".\"{}\" SET {}",
            self.schema,
            self.table,
            set_clauses.join(", ")
        );

        // WHERE 条件
        if !self.params.filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .params
                .filters
                .iter()
                .map(|filter| {
                    let condition = match filter.operator {
                        FilterOperator::Eq => {
                            args.add(&filter.value);
                            format!("\"{}\" = ${}", filter.column, arg_index)
                        }
                        _ => {
                            args.add(&filter.value);
                            format!("\"{}\" = ${}", filter.column, arg_index)
                        }
                    };
                    arg_index += 1;
                    condition
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" RETURNING *");

        Ok((sql, args))
    }

    /// 构建 DELETE 查询
    pub fn build_delete(&self) -> Result<(String, PgArguments)> {
        let mut args = PgArguments::default();
        let mut arg_index = 1;

        let mut sql = format!("DELETE FROM \"{}\".\"{}\"", self.schema, self.table);

        // WHERE 条件 (DELETE 必须有条件)
        if self.params.filters.is_empty() {
            return Err(AppError::InvalidQuery(
                "DELETE 操作必须提供 WHERE 条件".to_string(),
            ));
        }

        sql.push_str(" WHERE ");
        let conditions: Vec<String> = self
            .params
            .filters
            .iter()
            .map(|filter| {
                args.add(&filter.value);
                let condition = format!("\"{}\" = ${}", filter.column, arg_index);
                arg_index += 1;
                condition
            })
            .collect();
        sql.push_str(&conditions.join(" AND "));

        sql.push_str(" RETURNING *");

        Ok((sql, args))
    }
}

