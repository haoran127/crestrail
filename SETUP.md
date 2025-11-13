# CrestRail 快速部署指南

## 📋 前置要求

1. **Rust** (1.70+)
   ```bash
   # 安装 Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # 验证安装
   rustc --version
   cargo --version
   ```

2. **PostgreSQL** (12+)
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install postgresql postgresql-contrib
   
   # macOS
   brew install postgresql@14
   
   # Windows
   # 从 https://www.postgresql.org/download/windows/ 下载安装
   ```

## 🚀 快速开始

### 1. 克隆项目（如果适用）

```bash
git clone <your-repo-url>
cd crestrail
```

### 2. 配置数据库

```bash
# 启动 PostgreSQL
sudo systemctl start postgresql  # Linux
brew services start postgresql   # macOS

# 登录 PostgreSQL
sudo -u postgres psql

# 创建数据库和用户
CREATE DATABASE crestrail_db;
CREATE USER crestrail_user WITH ENCRYPTED PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE crestrail_db TO crestrail_user;

# 退出 psql
\q
```

### 3. 创建示例表

```bash
# 连接到数据库
psql -U crestrail_user -d crestrail_db

# 执行以下 SQL
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    age INTEGER,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 插入测试数据
INSERT INTO users (name, email, age, status) VALUES
('张三', 'zhangsan@example.com', 25, 'active'),
('李四', 'lisi@example.com', 30, 'active'),
('王五', 'wangwu@example.com', 22, 'pending');

-- 验证数据
SELECT * FROM users;

-- 退出
\q
```

### 4. 配置环境变量

创建 `.env` 文件：

```bash
# 复制示例配置
cat > .env << 'EOF'
DATABASE_URL=postgresql://crestrail_user:your_password@localhost:5432/crestrail_db
HOST=127.0.0.1
PORT=3000
RUST_LOG=info,crestrail=debug
EOF
```

**重要**: 将 `your_password` 替换为你实际设置的密码！

### 5. 运行项目

```bash
# 开发模式（自动重新编译）
cargo run

# 或者先编译再运行
cargo build
./target/debug/crestrail

# 生产模式（优化编译）
cargo build --release
./target/release/crestrail
```

你应该看到：

```
🚀 服务器启动在 http://127.0.0.1:3000
📡 API 端点: http://127.0.0.1:3000/api/:schema/:table
```

### 6. 测试 API

打开新终端，测试 API：

```bash
# 查询所有用户
curl http://localhost:3000/api/public/users

# 查询特定用户
curl "http://localhost:3000/api/public/users?id=1"

# 创建用户
curl -X POST http://localhost:3000/api/public/users \
  -H "Content-Type: application/json" \
  -d '{"name":"赵六","email":"zhaoliu@example.com","age":28}'

# 更新用户
curl -X PATCH "http://localhost:3000/api/public/users?id=1" \
  -H "Content-Type: application/json" \
  -d '{"status":"verified"}'

# 删除用户
curl -X DELETE "http://localhost:3000/api/public/users?id=1"
```

### 7. 测试前端示例

```bash
# 在浏览器中打开
# 方式 1: 直接用浏览器打开文件
open examples/frontend-demo.html  # macOS
xdg-open examples/frontend-demo.html  # Linux
start examples/frontend-demo.html  # Windows

# 方式 2: 使用简单 HTTP 服务器
cd examples
python3 -m http.server 8080
# 然后访问 http://localhost:8080/frontend-demo.html
```

## 🔧 常见问题

### 问题 1: 数据库连接失败

```
Error: database connection failed
```

**解决方案**:
1. 检查 PostgreSQL 是否运行：`sudo systemctl status postgresql`
2. 验证 `.env` 中的数据库 URL 是否正确
3. 测试数据库连接：`psql -U crestrail_user -d crestrail_db`

### 问题 2: 编译错误

```
error: could not compile `crestrail`
```

**解决方案**:
1. 更新 Rust：`rustup update`
2. 清理缓存：`cargo clean`
3. 重新编译：`cargo build`

### 问题 3: CORS 错误

```
CORS policy blocked
```

**解决方案**:
已配置 CORS 允许所有来源。如果仍有问题，检查浏览器控制台的具体错误信息。

### 问题 4: 端口被占用

```
Address already in use
```

**解决方案**:
1. 更改 `.env` 中的 `PORT` 值
2. 或者终止占用端口的进程：
   ```bash
   # Linux/macOS
   lsof -ti:3000 | xargs kill -9
   
   # Windows
   netstat -ano | findstr :3000
   taskkill /PID <PID> /F
   ```

## 📦 生产部署

### 使用 Docker（推荐）

创建 `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/crestrail /usr/local/bin/
EXPOSE 3000
CMD ["crestrail"]
```

创建 `docker-compose.yml`:

```yaml
version: '3.8'

services:
  db:
    image: postgres:14
    environment:
      POSTGRES_DB: crestrail_db
      POSTGRES_USER: crestrail_user
      POSTGRES_PASSWORD: your_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://crestrail_user:your_password@db:5432/crestrail_db
      HOST: 0.0.0.0
      PORT: 3000
    depends_on:
      - db

volumes:
  postgres_data:
```

运行：

```bash
docker-compose up -d
```

### 使用 Systemd（Linux）

创建服务文件 `/etc/systemd/system/crestrail.service`:

```ini
[Unit]
Description=CrestRail API Server
After=network.target postgresql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/crestrail
Environment="DATABASE_URL=postgresql://crestrail_user:password@localhost/crestrail_db"
Environment="HOST=0.0.0.0"
Environment="PORT=3000"
ExecStart=/opt/crestrail/target/release/crestrail
Restart=always

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable crestrail
sudo systemctl start crestrail
sudo systemctl status crestrail
```

### 使用 Nginx 反向代理

```nginx
server {
    listen 80;
    server_name api.yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## 🔒 安全建议

1. **生产环境**：
   - 使用强密码
   - 启用 SSL/TLS
   - 限制数据库用户权限
   - 使用防火墙限制访问

2. **数据库权限**：
   ```sql
   -- 只授予必要的权限
   REVOKE ALL ON DATABASE crestrail_db FROM crestrail_user;
   GRANT CONNECT ON DATABASE crestrail_db TO crestrail_user;
   GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO crestrail_user;
   ```

3. **环境变量**：
   - 永远不要提交 `.env` 文件到版本控制
   - 使用密钥管理服务（如 AWS Secrets Manager）

## 📊 性能优化

1. **数据库索引**：
   ```sql
   CREATE INDEX idx_users_status ON users(status);
   CREATE INDEX idx_users_created_at ON users(created_at);
   ```

2. **连接池大小**（在 `src/db.rs` 中调整）：
   ```rust
   PgPoolOptions::new()
       .max_connections(20)  // 根据负载调整
       .connect(database_url)
       .await?
   ```

3. **日志级别**：
   ```env
   # 生产环境使用 info 或 warn
   RUST_LOG=warn,crestrail=info
   ```

## 🆘 获取帮助

- 查看日志：`RUST_LOG=debug cargo run`
- 测试数据库连接：`psql -U crestrail_user -d crestrail_db`
- 检查端口：`netstat -tuln | grep 3000`

祝你使用愉快！🎉

