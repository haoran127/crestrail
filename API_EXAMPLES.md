# CrestRail API 使用示例大全

## 📌 基础概念

API 格式：`/api/:schema/:table`

- `schema`: PostgreSQL schema 名称（通常是 `public`）
- `table`: 表名

## 🔍 查询操作 (GET)

### 1. 基础查询

```bash
# 获取所有记录
curl "http://localhost:3000/api/public/users"

# 获取单条记录
curl "http://localhost:3000/api/public/users?id=1"

# 选择特定字段
curl "http://localhost:3000/api/public/users?select=id,name,email"
```

### 2. 过滤条件

#### 2.1 等于 (eq)

```bash
# 隐式等于
curl "http://localhost:3000/api/public/users?status=active"

# 显式等于
curl "http://localhost:3000/api/public/users?status.eq=active"
```

#### 2.2 不等于 (neq)

```bash
curl "http://localhost:3000/api/public/users?status.neq=inactive"
```

#### 2.3 大于/大于等于 (gt/gte)

```bash
# 大于
curl "http://localhost:3000/api/public/users?age.gt=18"

# 大于等于
curl "http://localhost:3000/api/public/users?age.gte=18"
```

#### 2.4 小于/小于等于 (lt/lte)

```bash
# 小于
curl "http://localhost:3000/api/public/users?age.lt=65"

# 小于等于
curl "http://localhost:3000/api/public/users?age.lte=65"
```

#### 2.5 模糊查询 (like/ilike)

```bash
# 区分大小写
curl "http://localhost:3000/api/public/users?name.like=%张%"

# 不区分大小写
curl "http://localhost:3000/api/public/users?name.ilike=%zhang%"
```

#### 2.6 IN 查询

```bash
curl "http://localhost:3000/api/public/users?status.in=active,verified,pending"
```

#### 2.7 NULL 查询

```bash
# IS NULL
curl "http://localhost:3000/api/public/users?deleted_at.is=null"

# IS NOT NULL
curl "http://localhost:3000/api/public/users?deleted_at.is=notnull"
```

### 3. 组合条件

```bash
# AND 条件（多个参数自动组合）
curl "http://localhost:3000/api/public/users?status=active&age.gte=18&age.lte=65"
```

### 4. 排序

```bash
# 单字段升序（默认）
curl "http://localhost:3000/api/public/users?order=created_at"
curl "http://localhost:3000/api/public/users?order=created_at.asc"

# 单字段降序
curl "http://localhost:3000/api/public/users?order=created_at.desc"

# 多字段排序
curl "http://localhost:3000/api/public/users?order=status.asc,created_at.desc"
```

### 5. 分页

```bash
# 限制返回数量
curl "http://localhost:3000/api/public/users?limit=10"

# 带偏移量
curl "http://localhost:3000/api/public/users?limit=10&offset=20"

# 实现分页（第 3 页，每页 10 条）
curl "http://localhost:3000/api/public/users?limit=10&offset=20"
```

### 6. 综合查询示例

```bash
# 查询活跃用户，年龄 18-65，按创建时间降序，前 20 条
curl "http://localhost:3000/api/public/users?select=id,name,email,age&status=active&age.gte=18&age.lte=65&order=created_at.desc&limit=20"

# 查询已发布的文章，浏览量大于 100，按浏览量降序
curl "http://localhost:3000/api/public/posts?status=published&views.gt=100&order=views.desc&limit=10"
```

## ➕ 创建操作 (POST)

### 1. 单条插入

```bash
curl -X POST "http://localhost:3000/api/public/users" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三",
    "email": "zhangsan@example.com",
    "age": 25,
    "status": "active"
  }'
```

### 2. 批量插入

```bash
curl -X POST "http://localhost:3000/api/public/users" \
  -H "Content-Type: application/json" \
  -d '[
    {
      "name": "张三",
      "email": "zhangsan@example.com",
      "age": 25
    },
    {
      "name": "李四",
      "email": "lisi@example.com",
      "age": 30
    },
    {
      "name": "王五",
      "email": "wangwu@example.com",
      "age": 22
    }
  ]'
```

### 3. 插入关联数据

```bash
# 先创建用户
curl -X POST "http://localhost:3000/api/public/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "作者", "email": "author@example.com"}'

# 然后创建文章（使用返回的 user_id）
curl -X POST "http://localhost:3000/api/public/posts" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 1,
    "title": "我的第一篇文章",
    "content": "文章内容...",
    "status": "published"
  }'
```

## 🔄 更新操作 (PATCH)

### 1. 单条更新

```bash
curl -X PATCH "http://localhost:3000/api/public/users?id=1" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三（已更新）",
    "age": 26
  }'
```

### 2. 批量更新

```bash
# 将所有 pending 状态的用户改为 active
curl -X PATCH "http://localhost:3000/api/public/users?status=pending" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "active"
  }'
```

### 3. 部分字段更新

```bash
# 只更新 status 字段
curl -X PATCH "http://localhost:3000/api/public/users?id=1" \
  -H "Content-Type: application/json" \
  -d '{"status": "verified"}'
```

### 4. 条件更新

```bash
# 更新特定条件的记录
curl -X PATCH "http://localhost:3000/api/public/posts?status=draft&views.lt=10" \
  -H "Content-Type: application/json" \
  -d '{"status": "archived"}'
```

## ❌ 删除操作 (DELETE)

### 1. 单条删除

```bash
curl -X DELETE "http://localhost:3000/api/public/users?id=1"
```

### 2. 批量删除

```bash
# 删除所有 inactive 的用户
curl -X DELETE "http://localhost:3000/api/public/users?status=inactive"
```

### 3. 条件删除

```bash
# 删除旧数据（比如创建时间早于某日期）
curl -X DELETE "http://localhost:3000/api/public/posts?status=archived&views.lt=5"
```

## 🎯 实际业务场景示例

### 场景 1: 用户管理

```bash
# 1. 注册新用户
curl -X POST "http://localhost:3000/api/public/users" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新用户",
    "email": "newuser@example.com",
    "status": "pending"
  }'

# 2. 验证用户
curl -X PATCH "http://localhost:3000/api/public/users?email=newuser@example.com" \
  -H "Content-Type: application/json" \
  -d '{"status": "verified"}'

# 3. 查询用户信息
curl "http://localhost:3000/api/public/users?email=newuser@example.com"

# 4. 软删除（标记删除时间）
curl -X PATCH "http://localhost:3000/api/public/users?email=newuser@example.com" \
  -H "Content-Type: application/json" \
  -d '{"deleted_at": "2024-01-01T00:00:00Z"}'
```

### 场景 2: 博客系统

```bash
# 1. 创建文章
curl -X POST "http://localhost:3000/api/public/posts" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 1,
    "title": "Rust 编程入门",
    "content": "Rust 是一门系统编程语言...",
    "status": "draft"
  }'

# 2. 发布文章
curl -X PATCH "http://localhost:3000/api/public/posts?id=1" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "published",
    "published_at": "2024-01-01T12:00:00Z"
  }'

# 3. 增加浏览量
curl -X PATCH "http://localhost:3000/api/public/posts?id=1" \
  -H "Content-Type: application/json" \
  -d '{"views": 101}'

# 4. 查询热门文章
curl "http://localhost:3000/api/public/posts?status=published&order=views.desc&limit=10"

# 5. 查询用户的文章
curl "http://localhost:3000/api/public/posts?user_id=1&order=created_at.desc"
```

### 场景 3: 评论系统

```bash
# 1. 添加评论
curl -X POST "http://localhost:3000/api/public/comments" \
  -H "Content-Type: application/json" \
  -d '{
    "post_id": 1,
    "user_id": 2,
    "content": "写得很好！"
  }'

# 2. 回复评论（嵌套评论）
curl -X POST "http://localhost:3000/api/public/comments" \
  -H "Content-Type: application/json" \
  -d '{
    "post_id": 1,
    "user_id": 1,
    "parent_id": 1,
    "content": "谢谢支持！"
  }'

# 3. 查询文章的所有评论
curl "http://localhost:3000/api/public/comments?post_id=1&order=created_at.asc"

# 4. 查询用户的所有评论
curl "http://localhost:3000/api/public/comments?user_id=2&order=created_at.desc"
```

### 场景 4: 标签系统

```bash
# 1. 创建标签
curl -X POST "http://localhost:3000/api/public/tags" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Rust",
    "description": "Rust 编程语言相关"
  }'

# 2. 给文章添加标签
curl -X POST "http://localhost:3000/api/public/post_tags" \
  -H "Content-Type: application/json" \
  -d '{
    "post_id": 1,
    "tag_id": 1
  }'

# 3. 查询标签的文章（通过关联表）
curl "http://localhost:3000/api/public/post_tags?tag_id=1"

# 4. 查询文章的标签
curl "http://localhost:3000/api/public/post_tags?post_id=1"
```

## 🌐 前端集成示例

### JavaScript Fetch

```javascript
// 封装 API 客户端
class CrestRailClient {
  constructor(baseUrl = 'http://localhost:3000/api') {
    this.baseUrl = baseUrl;
  }

  async get(schema, table, params = {}) {
    const url = new URL(`${this.baseUrl}/${schema}/${table}`);
    Object.entries(params).forEach(([key, value]) => {
      url.searchParams.append(key, value);
    });
    const response = await fetch(url);
    return response.json();
  }

  async create(schema, table, data) {
    const response = await fetch(`${this.baseUrl}/${schema}/${table}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return response.json();
  }

  async update(schema, table, params, data) {
    const url = new URL(`${this.baseUrl}/${schema}/${table}`);
    Object.entries(params).forEach(([key, value]) => {
      url.searchParams.append(key, value);
    });
    const response = await fetch(url, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return response.json();
  }

  async delete(schema, table, params) {
    const url = new URL(`${this.baseUrl}/${schema}/${table}`);
    Object.entries(params).forEach(([key, value]) => {
      url.searchParams.append(key, value);
    });
    const response = await fetch(url, { method: 'DELETE' });
    return response.json();
  }
}

// 使用示例
const api = new CrestRailClient();

// 查询
const users = await api.get('public', 'users', {
  status: 'active',
  'age.gte': 18,
  order: 'created_at.desc',
  limit: 10,
});

// 创建
const newUser = await api.create('public', 'users', {
  name: '张三',
  email: 'zhangsan@example.com',
});

// 更新
const updated = await api.update(
  'public',
  'users',
  { id: 1 },
  { status: 'verified' }
);

// 删除
await api.delete('public', 'users', { id: 1 });
```

## 🔐 高级用法

### 1. URL 编码

```bash
# 空格和特殊字符需要编码
curl "http://localhost:3000/api/public/users?name.like=%E5%BC%A0%25"  # 张%
```

### 2. 复杂查询组合

```bash
# 查询活跃用户，年龄 20-30，邮箱包含 gmail，按年龄升序，每页 20 条
curl "http://localhost:3000/api/public/users?status=active&age.gte=20&age.lte=30&email.ilike=%gmail%&order=age.asc&limit=20&offset=0"
```

### 3. JSON 格式化输出

```bash
# 使用 jq 格式化 JSON
curl "http://localhost:3000/api/public/users" | jq '.'

# 提取特定字段
curl "http://localhost:3000/api/public/users" | jq '.[].name'

# 统计数量
curl "http://localhost:3000/api/public/users?status=active" | jq 'length'
```

## 📊 性能优化建议

1. **使用 select 限制字段**: 只查询需要的字段
   ```bash
   curl "http://localhost:3000/api/public/users?select=id,name"
   ```

2. **合理使用 limit**: 避免一次性查询大量数据
   ```bash
   curl "http://localhost:3000/api/public/users?limit=100"
   ```

3. **使用索引字段过滤**: 在有索引的字段上进行过滤
   ```bash
   curl "http://localhost:3000/api/public/users?id=1"  # id 有主键索引
   ```

4. **批量操作**: 使用批量插入替代多次单条插入
   ```bash
   curl -X POST "http://localhost:3000/api/public/users" \
     -H "Content-Type: application/json" \
     -d '[{...}, {...}, {...}]'
   ```

## ❗ 常见错误处理

### 错误响应格式

```json
{
  "error": "错误描述信息"
}
```

### 常见错误

1. **400 Bad Request**: 查询参数错误或 JSON 格式错误
2. **500 Internal Server Error**: 数据库错误或服务器错误

查看服务器日志获取详细错误信息。

---

更多信息请参考 [README.md](README.md) 和 [SETUP.md](SETUP.md)

