# CrestRail Next.js 快速启动指南

## 🚀 启动步骤

### 1. 安装依赖

```bash
cd frontend-nextjs
npm install
```

### 2. 启动开发服务器

```bash
npm run dev
```

访问：**http://localhost:3001** （Next.js 默认端口为 3000，如果被占用会自动使用 3001）

### 3. 确保后端运行

在另一个终端窗口：

```bash
cd ..
cargo run
```

后端应该运行在：**http://localhost:3000**

## 🔧 故障排查

### 问题 1：看不到数据返回结果

**原因**：前端请求后端 API 失败

**解决方案**：

1. **打开浏览器开发者工具**（F12）
2. 切换到 **Console** 标签页，查看是否有错误
3. 切换到 **Network** 标签页，查看 API 请求是否成功

### 问题 2：API 请求 404

**检查清单**：

- [ ] 后端是否正在运行？（`cargo run`）
- [ ] 后端端口是否是 3000？
- [ ] 前端 API 路径是否正确？

**测试后端是否正常**：

```bash
# 测试健康检查
curl http://localhost:3000/health

# 测试登录
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"Admin123"}'
```

### 问题 3：CORS 错误

**症状**：浏览器控制台显示类似错误：
```
Access to XMLHttpRequest at 'http://localhost:3000/...' from origin 'http://localhost:3001' has been blocked by CORS policy
```

**解决方案**：后端已经配置了 CORS，如果还有问题，检查 `src/main.rs`:

```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

### 问题 4：登录失败

**检查数据库**：

```sql
-- 检查用户是否存在
SELECT * FROM public.users;

-- 如果没有用户，创建一个
INSERT INTO public.users (email, password_hash, role)
VALUES ('admin@example.com', '$2b$12$...', 'admin');
```

**或者使用注册接口**：

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"Admin123"}'
```

## 📊 调试技巧

### 1. 查看网络请求

打开浏览器开发者工具 → Network 标签页：

- **红色请求**：失败的请求，点击查看详情
- **200 状态码**：成功
- **401 状态码**：未授权，token 可能过期
- **404 状态码**：路径不存在
- **500 状态码**：服务器错误

### 2. 查看控制台日志

前端日志会显示在浏览器控制台：

```javascript
console.log('API Response:', response)
console.error('Error:', error)
```

### 3. 查看后端日志

后端日志会显示在运行 `cargo run` 的终端：

```
2024-11-14T12:00:00.000Z INFO  crestrail::main: 配置加载成功
2024-11-14T12:00:00.001Z INFO  crestrail::main: 数据库连接成功
2024-11-14T12:00:00.002Z INFO  crestrail::main: 服务器启动: http://127.0.0.1:3000
```

### 4. 测试 API 端点

使用 curl 或 Postman 测试：

```bash
# 1. 登录获取 token
TOKEN=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"Admin123"}' \
  | jq -r '.token')

echo "Token: $TOKEN"

# 2. 测试受保护的端点
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/schemas

# 3. 测试 SQL 查询
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"sql":"SELECT * FROM public.users LIMIT 5"}'
```

## 🎯 常见端点测试

### 公开端点（不需要 token）

```bash
# 健康检查
curl http://localhost:3000/health

# 登录
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"Admin123"}'

# 注册
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123"}'
```

### 需要认证的端点（需要 token）

```bash
# 先获取 token（替换为你的实际 token）
export TOKEN="your_jwt_token_here"

# 获取 schemas
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/schemas

# 获取表列表
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/schema/public/tables

# 获取表结构
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/schema/public/table/users/structure

# 查询数据
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/public/users?limit=10"

# 执行 SQL
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"sql":"SELECT version();"}'
```

## 💡 开发提示

1. **热重载**：修改代码后，Next.js 会自动重新加载
2. **查看编译错误**：终端会显示 TypeScript 类型错误
3. **清除缓存**：如果遇到奇怪的问题，尝试：
   ```bash
   rm -rf .next node_modules
   npm install
   npm run dev
   ```

## 📱 浏览器兼容性

推荐使用：
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## 🔑 默认测试账号

如果数据库中没有用户，请先注册或手动插入：

```sql
-- 密码是 "Admin123" 的 bcrypt hash
INSERT INTO public.users (email, password_hash, role, created_at, updated_at)
VALUES (
  'admin@example.com',
  '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5Zm1Q4n.ZJBz2',
  'admin',
  NOW(),
  NOW()
);
```

## ❓ 还是不行？

如果按照上面的步骤还是无法正常运行，请检查：

1. **Node.js 版本**：需要 18.17+ 或 20+
   ```bash
   node --version
   ```

2. **Rust 后端是否正常运行**
   ```bash
   curl http://localhost:3000/health
   ```

3. **数据库连接**：检查 `.env` 文件中的 `DATABASE_URL`

4. **防火墙**：确保 3000 和 3001 端口没有被防火墙阻止

5. **查看完整日志**：
   - 前端：浏览器开发者工具 Console
   - 后端：运行 `cargo run` 的终端输出

