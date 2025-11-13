# CrestRail 架构设计文档

## 📐 整体架构

```
┌─────────────┐
│   前端应用   │ (React/Vue/任意前端框架)
│  (Browser)  │
└──────┬──────┘
       │ HTTP REST API
       │
┌──────▼──────────────────────────────────────┐
│           CrestRail API Server              │
│         (Rust + Axum + SQLx)                │
│                                             │
│  ┌─────────────────────────────────────┐   │
│  │        HTTP 层 (Axum)               │   │
│  │  - 路由管理                          │   │
│  │  - CORS 中间件                       │   │
│  │  - 请求/响应处理                     │   │
│  └──────────┬──────────────────────────┘   │
│             │                               │
│  ┌──────────▼──────────────────────────┐   │
│  │     业务逻辑层 (Handlers)            │   │
│  │  - CRUD 处理器                       │   │
│  │  - 参数验证                          │   │
│  │  - 错误处理                          │   │
│  └──────────┬──────────────────────────┘   │
│             │                               │
│  ┌──────────▼──────────────────────────┐   │
│  │  查询构建层 (Query Builder)          │   │
│  │  - URL 参数解析                      │   │
│  │  - SQL 生成                          │   │
│  │  - 参数绑定                          │   │
│  │  - 安全验证                          │   │
│  └──────────┬──────────────────────────┘   │
│             │                               │
│  ┌──────────▼──────────────────────────┐   │
│  │      数据访问层 (SQLx)               │   │
│  │  - 连接池管理                        │   │
│  │  - SQL 执行                          │   │
│  │  - 结果映射                          │   │
│  └──────────┬──────────────────────────┘   │
└─────────────┼───────────────────────────────┘
              │ PostgreSQL Wire Protocol
              │
     ┌────────▼────────┐
     │   PostgreSQL    │
     │    Database     │
     └─────────────────┘
```

## 📦 模块设计

### 1. main.rs - 应用入口

**职责**:
- 初始化日志系统
- 加载配置
- 创建数据库连接池
- 配置 CORS
- 注册路由
- 启动 HTTP 服务器

**关键代码**:
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 日志初始化
    tracing_subscriber::registry()...;
    
    // 配置加载
    let config = Config::from_env()?;
    
    // 数据库连接池
    let pool = db::create_pool(&config.database_url).await?;
    
    // 路由注册
    let app = Router::new()
        .route("/api/:schema/:table", get(handlers::get_records))
        .route("/api/:schema/:table", post(handlers::create_record))
        .route("/api/:schema/:table", patch(handlers::update_records))
        .route("/api/:schema/:table", delete(handlers::delete_records))
        .with_state(pool)
        .layer(cors);
    
    // 启动服务器
    axum::serve(listener, app).await?;
}
```

### 2. config.rs - 配置管理

**职责**:
- 从环境变量读取配置
- 提供配置结构体
- 配置验证

**配置项**:
- `DATABASE_URL`: 数据库连接字符串
- `HOST`: 服务器监听地址
- `PORT`: 服务器监听端口
- `RUST_LOG`: 日志级别

### 3. db.rs - 数据库连接

**职责**:
- 创建数据库连接池
- 管理连接生命周期

**连接池配置**:
- 最大连接数: 10（可调整）
- 连接超时
- 空闲连接回收

### 4. error.rs - 错误处理

**职责**:
- 定义统一的错误类型
- 错误到 HTTP 响应的转换
- 错误日志记录

**错误类型**:
```rust
pub enum AppError {
    Database(sqlx::Error),      // 数据库错误
    InvalidQuery(String),        // 无效查询参数
    InvalidJson(serde_json::Error), // JSON 解析错误
    Internal(String),            // 内部错误
}
```

**响应格式**:
```json
{
  "error": "错误描述信息"
}
```

### 5. query_builder.rs - 核心查询构建器 ⭐

这是系统的核心模块，负责将 URL 查询参数安全地转换为 SQL。

#### 5.1 QueryParams - 参数解析

**输入**: URL 查询参数（HashMap）
```
status=active&age.gte=18&order=created_at.desc&limit=10
```

**输出**: 结构化的查询参数
```rust
QueryParams {
    filters: vec![
        Filter { column: "status", operator: Eq, value: "active" },
        Filter { column: "age", operator: Gte, value: "18" },
    ],
    order_by: vec![
        OrderBy { column: "created_at", ascending: false },
    ],
    limit: Some(10),
    offset: None,
    select: None,
}
```

#### 5.2 过滤操作符支持

| 操作符 | 语法 | SQL | 示例 |
|--------|------|-----|------|
| 等于 | `field=value` | `=` | `status=active` |
| 显式等于 | `field.eq=value` | `=` | `status.eq=active` |
| 不等于 | `field.neq=value` | `!=` | `status.neq=inactive` |
| 大于 | `field.gt=value` | `>` | `age.gt=18` |
| 大于等于 | `field.gte=value` | `>=` | `age.gte=18` |
| 小于 | `field.lt=value` | `<` | `age.lt=65` |
| 小于等于 | `field.lte=value` | `<=` | `age.lte=65` |
| 模糊匹配 | `field.like=value` | `LIKE` | `name.like=%张%` |
| 不区分大小写 | `field.ilike=value` | `ILIKE` | `name.ilike=%zhang%` |
| IN 查询 | `field.in=v1,v2,v3` | `IN` | `status.in=active,pending` |
| NULL 查询 | `field.is=null` | `IS NULL` | `deleted_at.is=null` |

#### 5.3 SqlBuilder - SQL 生成

**SELECT 查询生成流程**:

1. **基础 SELECT**
   ```sql
   SELECT * FROM "schema"."table"
   ```

2. **字段选择**
   ```sql
   SELECT id, name, email FROM "schema"."table"
   ```

3. **WHERE 条件**
   ```sql
   SELECT * FROM "schema"."table"
   WHERE "status" = $1 AND "age" >= $2
   ```

4. **ORDER BY**
   ```sql
   SELECT * FROM "schema"."table"
   WHERE ...
   ORDER BY "created_at" DESC, "name" ASC
   ```

5. **LIMIT/OFFSET**
   ```sql
   SELECT * FROM "schema"."table"
   WHERE ...
   ORDER BY ...
   LIMIT $3 OFFSET $4
   ```

**安全机制**:

1. **标识符验证**
   ```rust
   fn sanitize_identifier(ident: &str) -> Result<String> {
       // 只允许字母、数字、下划线
       // 不允许以数字开头
       // 防止 SQL 注入
   }
   ```

2. **参数化查询**
   ```rust
   // ❌ 不安全（字符串拼接）
   let sql = format!("SELECT * FROM users WHERE id = {}", user_id);
   
   // ✅ 安全（参数绑定）
   let sql = "SELECT * FROM users WHERE id = $1";
   sqlx::query_with(sql, args.add(user_id))
   ```

3. **标识符引号包裹**
   ```sql
   -- 防止关键字冲突和大小写敏感问题
   SELECT "user", "order" FROM "public"."table"
   ```

### 6. handlers.rs - 请求处理器

#### 6.1 GET - 查询记录

```rust
pub async fn get_records(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>>
```

**流程**:
1. 解析查询参数
2. 构建 SQL
3. 执行查询
4. 转换结果为 JSON
5. 返回数组

#### 6.2 POST - 创建记录

```rust
pub async fn create_record(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Json(data): Json<Value>,
) -> Result<(StatusCode, Json<Value>)>
```

**特性**:
- 支持单条插入
- 支持批量插入（数组）
- 返回插入的记录（RETURNING *）
- 返回 201 Created 状态码

#### 6.3 PATCH - 更新记录

```rust
pub async fn update_records(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
    Json(data): Json<Value>,
) -> Result<Json<Value>>
```

**特性**:
- 必须提供 WHERE 条件（通过查询参数）
- 支持批量更新
- 返回更新后的记录（RETURNING *）

#### 6.4 DELETE - 删除记录

```rust
pub async fn delete_records(
    State(pool): State<PgPool>,
    Path((schema, table)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<(StatusCode, Json<Value>)>
```

**安全措施**:
- **必须**提供 WHERE 条件（防止误删全表）
- 支持批量删除
- 返回被删除的记录（RETURNING *）

## 🔒 安全设计

### 1. SQL 注入防护

#### 防护措施

**1.1 标识符验证**
```rust
fn sanitize_identifier(ident: &str) -> Result<String> {
    // 正则验证: ^[a-zA-Z_][a-zA-Z0-9_]*$
    if !ident.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AppError::InvalidQuery("无效的标识符".to_string()));
    }
    Ok(ident.to_string())
}
```

**测试案例**:
```
✅ "users" -> OK
✅ "user_profiles" -> OK
✅ "_internal" -> OK
❌ "users; DROP TABLE" -> Error
❌ "users--" -> Error
❌ "../etc/passwd" -> Error
```

**1.2 参数化查询**
```rust
// 所有值都通过参数绑定
args.add(&filter.value);
let sql = format!("WHERE \"{}\" = ${}", filter.column, arg_index);
```

**1.3 双引号包裹标识符**
```sql
-- 防止关键字注入
SELECT * FROM "public"."users" WHERE "order" = $1
```

### 2. 权限控制建议

虽然 API 本身不实现认证，但可以通过以下方式控制权限：

**2.1 数据库层面**
```sql
-- 创建只读用户
CREATE USER readonly_user WITH PASSWORD 'password';
GRANT CONNECT ON DATABASE mydb TO readonly_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_user;

-- 创建受限用户
CREATE USER app_user WITH PASSWORD 'password';
GRANT SELECT, INSERT, UPDATE ON specific_table TO app_user;
```

**2.2 行级安全策略 (RLS)**
```sql
-- 启用 RLS
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- 创建策略：用户只能看到自己的数据
CREATE POLICY user_isolation ON users
    USING (id = current_setting('app.user_id')::integer);
```

**2.3 反向代理层**
```nginx
# Nginx 认证
location /api/ {
    auth_request /auth;
    proxy_pass http://crestrail:3000;
}
```

### 3. 输入验证

```rust
// 1. 类型验证（Serde 自动处理）
#[derive(Deserialize)]
struct User {
    name: String,        // 必须是字符串
    age: Option<i32>,    // 可选整数
}

// 2. 长度限制（数据库约束）
CREATE TABLE users (
    name VARCHAR(100),   -- 最大 100 字符
    email VARCHAR(255)   -- 最大 255 字符
);

// 3. 格式验证（可扩展）
// 可以在 handler 中添加额外验证
if !email.contains('@') {
    return Err(AppError::InvalidQuery("无效的邮箱".to_string()));
}
```

## 🚀 性能优化

### 1. 连接池

```rust
PgPoolOptions::new()
    .max_connections(20)           // 最大连接数
    .min_connections(2)            // 最小连接数
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(database_url)
    .await?
```

### 2. 数据库索引

```sql
-- 常用查询字段加索引
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);

-- 复合索引
CREATE INDEX idx_users_status_created ON users(status, created_at);
```

### 3. 查询优化

```rust
// 1. 限制返回字段
?select=id,name,email

// 2. 使用 limit
?limit=100

// 3. 使用索引字段过滤
?id=1  // 主键索引
?email=test@example.com  // 唯一索引
```

### 4. 异步处理

```rust
// Axum + Tokio 实现异步 I/O
// 单线程可处理数千并发连接
#[tokio::main]
async fn main() {
    // 异步处理请求
    let app = Router::new()
        .route("/", get(async_handler));
}
```

## 📊 数据流示例

### 查询请求完整流程

**请求**:
```http
GET /api/public/users?status=active&age.gte=18&order=created_at.desc&limit=10
```

**1. Axum 路由匹配**
```rust
Path((schema, table)) = ("public", "users")
Query(query) = HashMap {
    "status": "active",
    "age.gte": "18",
    "order": "created_at.desc",
    "limit": "10"
}
```

**2. 参数解析**
```rust
QueryParams {
    filters: [
        Filter { column: "status", op: Eq, value: "active" },
        Filter { column: "age", op: Gte, value: "18" },
    ],
    order_by: [OrderBy { column: "created_at", ascending: false }],
    limit: Some(10),
    ...
}
```

**3. SQL 生成**
```sql
SELECT * FROM "public"."users"
WHERE "status" = $1 AND "age" >= $2
ORDER BY "created_at" DESC
LIMIT $3

参数: ["active", "18", 10]
```

**4. 数据库执行**
```
PostgreSQL -> 执行查询 -> 返回行
```

**5. 结果转换**
```rust
Vec<PgRow> -> Vec<serde_json::Value>
```

**6. HTTP 响应**
```json
[
  {
    "id": 5,
    "name": "张三",
    "email": "zhangsan@example.com",
    "age": 25,
    "status": "active",
    "created_at": "2024-01-01T12:00:00Z"
  },
  ...
]
```

## 🔮 迭代扩展方向（面向完整商业产品）

### 第一阶段：基础完善（1-2 周）

#### 1. 认证系统

```rust
// 添加 JWT 中间件
use axum_extra::extract::cookie::CookieJar;

async fn auth_middleware(
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Result<Response> {
    let token = jar.get("token")
        .ok_or(AppError::Unauthorized)?;
    
    verify_jwt(token)?;
    Ok(next.run(request).await)
}
```

#### 2. 数据验证

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(email)]
    email: String,
}

async fn validate_input<T: Validate>(data: &T) -> Result<(), AppError> {
    data.validate().map_err(|e| AppError::InvalidQuery(e.to_string()))
}
```

### 第二阶段：功能增强（2-4 周）

#### 3. 细粒度权限控制

```rust
// 行级安全策略
pub struct RLSPolicy {
    table: String,
    rule: String,  // SQL 表达式
}

// 在查询时注入策略
impl SqlBuilder {
    fn apply_rls(&mut self, user: &User) -> Result<()> {
        let policy = get_policy(&self.table, &user.role)?;
        self.filters.push(Filter::from_sql(&policy.rule));
        Ok(())
    }
}

// 列级权限
pub struct ColumnPermissions {
    table: String,
    role: String,
    allowed_columns: Vec<String>,
}
```

#### 4. 事务支持

```rust
// 事务 API 端点
pub async fn execute_transaction(
    State(pool): State<PgPool>,
    Json(ops): Json<Vec<Operation>>,
) -> Result<Json<Value>> {
    let mut tx = pool.begin().await?;
    
    let mut results = Vec::new();
    for op in ops {
        let result = match op.method {
            "POST" => insert_with_tx(&mut tx, &op).await?,
            "PATCH" => update_with_tx(&mut tx, &op).await?,
            "DELETE" => delete_with_tx(&mut tx, &op).await?,
            _ => return Err(AppError::InvalidQuery("无效操作".to_string())),
        };
        results.push(result);
    }
    
    tx.commit().await?;
    Ok(Json(json!(results)))
}
```

#### 5. 多表 JOIN 查询

```rust
// 解析嵌套查询语法: users?select=*,orders(*)
pub struct NestedQuery {
    fields: Vec<String>,
    relations: Vec<Relation>,
}

pub struct Relation {
    table: String,
    foreign_key: String,
    fields: Vec<String>,
}

impl SqlBuilder {
    fn build_join_query(&self) -> Result<String> {
        // 生成带 JOIN 的 SQL
        let mut sql = format!("SELECT {} FROM \"{}\".\"{}", 
            self.select_clause(),
            self.schema,
            self.table
        );
        
        for rel in &self.relations {
            sql.push_str(&format!(
                " LEFT JOIN \"{}\" ON \"{}\".\"{}\" = \"{}\".\"id\"",
                rel.table, self.table, rel.foreign_key, rel.table
            ));
        }
        
        Ok(sql)
    }
}
```

#### 6. 缓存层

```rust
use redis::AsyncCommands;

pub struct CacheManager {
    redis: redis::Client,
}

impl CacheManager {
    async fn get_with_cache(&self, key: &str, pool: &PgPool) -> Result<Value> {
        let mut conn = self.redis.get_async_connection().await?;
        
        // 先查缓存
        if let Some(cached) = conn.get::<_, String>(key).await.ok() {
            return Ok(serde_json::from_str(&cached)?);
        }
        
        // 缓存未命中，查数据库
        let result = query_database(pool).await?;
        
        // 写入缓存（1小时过期）
        conn.set_ex(key, serde_json::to_string(&result)?, 3600).await?;
        
        Ok(result)
    }
    
    async fn invalidate(&self, pattern: &str) -> Result<()> {
        // 失效相关缓存
        let mut conn = self.redis.get_async_connection().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;
        for key in keys {
            conn.del(&key).await?;
        }
        Ok(())
    }
}
```

### 第三阶段：企业级特性（1-2 月）

#### 7. 复杂业务逻辑引擎

```rust
// RPC 调用数据库函数
pub async fn call_rpc(
    State(pool): State<PgPool>,
    Path(function_name): Path<String>,
    Json(params): Json<Value>,
) -> Result<Json<Value>> {
    validate_function_name(&function_name)?;
    
    let sql = format!("SELECT * FROM \"{}\"($1)", function_name);
    let result = sqlx::query(&sql)
        .bind(&params)
        .fetch_all(&pool)
        .await?;
    
    Ok(Json(rows_to_json(result)?))
}

// Webhook 触发器
pub struct WebhookManager {
    hooks: HashMap<String, Vec<WebhookConfig>>,
}

impl WebhookManager {
    async fn trigger(&self, event: &str, data: &Value) {
        if let Some(hooks) = self.hooks.get(event) {
            for hook in hooks {
                let client = reqwest::Client::new();
                let _ = client.post(&hook.url)
                    .json(data)
                    .send()
                    .await;
            }
        }
    }
}

// 业务流程编排
pub struct WorkflowEngine {
    steps: Vec<WorkflowStep>,
}

pub enum WorkflowStep {
    Query { table: String, conditions: Value },
    Validate { rules: Vec<ValidationRule> },
    Transform { function: String },
    Save { table: String, data: Value },
}
```

#### 8. WebSocket 实时推送

```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use tokio::sync::broadcast;

pub struct RealtimeManager {
    channels: HashMap<String, broadcast::Sender<Value>>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(channel): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, channel))
}

async fn handle_socket(mut socket: WebSocket, channel: String) {
    let rx = subscribe_to_channel(&channel);
    
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg.to_string())).await.is_err() {
            break;
        }
    }
}
```

### 第四阶段：云原生和扩展（3-6 月）

#### 9. 读写分离

```rust
pub struct DatabaseCluster {
    master: PgPool,
    replicas: Vec<PgPool>,
}

impl DatabaseCluster {
    pub fn get_read_pool(&self) -> &PgPool {
        // 轮询选择只读副本
        &self.replicas[rand::random::<usize>() % self.replicas.len()]
    }
    
    pub fn get_write_pool(&self) -> &PgPool {
        &self.master
    }
}
```

#### 10. GraphQL 支持

```rust
use async_graphql::{Schema, Object};

#[Object]
impl Query {
    async fn users(&self, status: Option<String>) -> Vec<User> {
        // 复用现有的查询构建器
    }
}
```

#### 11. 插件系统

```rust
pub trait Middleware: Send + Sync {
    async fn before_query(&self, query: &mut QueryParams) -> Result<()>;
    async fn after_query(&self, result: &mut Value) -> Result<()>;
}

pub struct PluginManager {
    middlewares: Vec<Box<dyn Middleware>>,
}
```

## 📈 监控和日志

### 日志级别

```env
# 开发环境
RUST_LOG=debug,crestrail=trace,sqlx=debug

# 生产环境
RUST_LOG=info,crestrail=info,sqlx=warn
```

### 日志输出

```rust
tracing::info!("服务器启动");
tracing::debug!("执行 SQL: {}", sql);
tracing::error!("数据库错误: {}", err);
```

### 性能监控

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_http());
```

---

## 📚 参考资源

- [Axum 文档](https://docs.rs/axum)
- [SQLx 文档](https://docs.rs/sqlx)
- [PostgreSQL 文档](https://www.postgresql.org/docs/)
- [PostgREST](https://postgrest.org/) - 设计灵感来源

