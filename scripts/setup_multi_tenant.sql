-- ============================================
-- 多租户功能配置脚本
-- ============================================
-- 这个脚本会：
-- 1. 创建 management schema 和相关表
-- 2. 为 users 表添加角色字段
-- 3. 设置初始超级管理员
-- 4. 创建示例租户和连接

-- ============================================
-- 第一步：创建 management schema
-- ============================================

CREATE SCHEMA IF NOT EXISTS management;

-- 租户表
CREATE TABLE IF NOT EXISTS management.tenants (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, suspended, deleted
    contact_email VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 租户数据库连接配置表
CREATE TABLE IF NOT EXISTS management.tenant_databases (
    id SERIAL PRIMARY KEY,
    tenant_id INTEGER NOT NULL REFERENCES management.tenants(id) ON DELETE CASCADE,
    connection_name VARCHAR(100) NOT NULL,
    db_host VARCHAR(255) NOT NULL,
    db_port INTEGER NOT NULL DEFAULT 5432,
    db_name VARCHAR(100) NOT NULL,
    db_user VARCHAR(100) NOT NULL,
    db_password_encrypted TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    max_connections INTEGER DEFAULT 10,
    connection_timeout INTEGER DEFAULT 30,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, connection_name)
);

-- 租户业务 Schema 配置表
CREATE TABLE IF NOT EXISTS management.tenant_schemas (
    id SERIAL PRIMARY KEY,
    tenant_id INTEGER NOT NULL REFERENCES management.tenants(id) ON DELETE CASCADE,
    database_id INTEGER NOT NULL REFERENCES management.tenant_databases(id) ON DELETE CASCADE,
    schema_name VARCHAR(100) NOT NULL,
    business_type VARCHAR(100) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(database_id, schema_name)
);

-- 用户-租户关联表
CREATE TABLE IF NOT EXISTS management.user_tenants (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id INTEGER NOT NULL REFERENCES management.tenants(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member', -- owner, admin, member, viewer
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, tenant_id)
);

-- 连接访问日志表
CREATE TABLE IF NOT EXISTS management.connection_access_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    tenant_id INTEGER REFERENCES management.tenants(id),
    database_id INTEGER REFERENCES management.tenant_databases(id),
    action VARCHAR(50) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_tenant_databases_tenant_id ON management.tenant_databases(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_databases_active ON management.tenant_databases(is_active);
CREATE INDEX IF NOT EXISTS idx_tenant_schemas_tenant_id ON management.tenant_schemas(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_schemas_database_id ON management.tenant_schemas(database_id);
CREATE INDEX IF NOT EXISTS idx_user_tenants_user_id ON management.user_tenants(user_id);
CREATE INDEX IF NOT EXISTS idx_user_tenants_tenant_id ON management.user_tenants(tenant_id);
CREATE INDEX IF NOT EXISTS idx_connection_logs_tenant_id ON management.connection_access_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_connection_logs_created_at ON management.connection_access_logs(created_at);

-- 创建更新时间触发器
CREATE OR REPLACE FUNCTION management.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tenants_updated_at 
    BEFORE UPDATE ON management.tenants
    FOR EACH ROW EXECUTE FUNCTION management.update_updated_at_column();

CREATE TRIGGER update_tenant_databases_updated_at 
    BEFORE UPDATE ON management.tenant_databases
    FOR EACH ROW EXECUTE FUNCTION management.update_updated_at_column();

CREATE TRIGGER update_tenant_schemas_updated_at 
    BEFORE UPDATE ON management.tenant_schemas
    FOR EACH ROW EXECUTE FUNCTION management.update_updated_at_column();

-- ============================================
-- 第二步：扩展 users 表，添加全局角色字段
-- ============================================

-- 添加 role 字段（如果不存在）
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'role'
    ) THEN
        ALTER TABLE users ADD COLUMN role VARCHAR(50) NOT NULL DEFAULT 'tenant_user';
        -- role 可选值: superadmin, tenant_admin, tenant_user
    END IF;
END $$;

-- 添加 is_active 字段（如果不存在）
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'is_active'
    ) THEN
        ALTER TABLE users ADD COLUMN is_active BOOLEAN DEFAULT true;
    END IF;
END $$;

-- ============================================
-- 第三步：设置初始超级管理员
-- ============================================

-- 将 admin 用户设置为超级管理员
UPDATE users 
SET role = 'superadmin', is_active = true
WHERE username = 'admin' OR email = 'admin@example.com';

-- 如果没有 admin 用户，创建一个
INSERT INTO users (username, email, password_hash, role, is_active)
SELECT 
    'superadmin',
    'superadmin@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5jmJ6n8QhJvqK', -- 密码: admin123
    'superadmin',
    true
WHERE NOT EXISTS (
    SELECT 1 FROM users WHERE role = 'superadmin'
);

-- ============================================
-- 第四步：创建实用视图
-- ============================================

-- 租户完整信息视图
CREATE OR REPLACE VIEW management.v_tenant_info AS
SELECT 
    t.id AS tenant_id,
    t.name AS tenant_name,
    t.slug AS tenant_slug,
    t.status,
    COUNT(DISTINCT td.id) AS database_count,
    COUNT(DISTINCT ts.id) AS schema_count,
    COUNT(DISTINCT ut.user_id) AS user_count,
    t.created_at
FROM management.tenants t
LEFT JOIN management.tenant_databases td ON td.tenant_id = t.id AND td.is_active = true
LEFT JOIN management.tenant_schemas ts ON ts.tenant_id = t.id AND ts.is_active = true
LEFT JOIN management.user_tenants ut ON ut.tenant_id = t.id AND ut.is_active = true
GROUP BY t.id, t.name, t.slug, t.status, t.created_at;

-- 用户可访问的连接视图
CREATE OR REPLACE VIEW management.v_user_connections AS
SELECT 
    u.id AS user_id,
    u.username,
    t.id AS tenant_id,
    t.name AS tenant_name,
    td.id AS database_id,
    td.connection_name,
    td.db_host,
    td.db_port,
    td.db_name,
    td.is_primary,
    ut.role AS user_role
FROM users u
JOIN management.user_tenants ut ON ut.user_id = u.id AND ut.is_active = true
JOIN management.tenants t ON t.id = ut.tenant_id AND t.status = 'active'
JOIN management.tenant_databases td ON td.tenant_id = t.id AND td.is_active = true;

-- ============================================
-- 第五步：插入示例数据（可选）
-- ============================================

-- 创建示例租户
INSERT INTO management.tenants (name, slug, contact_email, status) VALUES
    ('演示公司 A', 'demo-company-a', 'contact@company-a.com', 'active'),
    ('演示公司 B', 'demo-company-b', 'contact@company-b.com', 'active')
ON CONFLICT (slug) DO NOTHING;

-- 为演示租户创建数据库连接（指向当前数据库）
INSERT INTO management.tenant_databases (tenant_id, connection_name, db_host, db_port, db_name, db_user, db_password_encrypted, is_primary)
SELECT 
    t.id,
    '主数据库',
    'localhost',
    5432,
    current_database(), -- 使用当前数据库名
    current_user, -- 使用当前用户
    'ENCRYPTED:changeme', -- 实际使用时需要真实密码
    true
FROM management.tenants t
WHERE t.slug IN ('demo-company-a', 'demo-company-b')
ON CONFLICT (tenant_id, connection_name) DO NOTHING;

-- 为示例租户创建 schema 配置
INSERT INTO management.tenant_schemas (tenant_id, database_id, schema_name, business_type, display_name, description)
SELECT 
    t.id,
    td.id,
    'public',
    'default',
    '默认 Schema',
    '默认的 public schema'
FROM management.tenants t
JOIN management.tenant_databases td ON td.tenant_id = t.id
WHERE t.slug IN ('demo-company-a', 'demo-company-b')
ON CONFLICT (database_id, schema_name) DO NOTHING;

-- ============================================
-- 完成提示
-- ============================================

SELECT '✅ 多租户功能配置完成！' AS status,
       '已创建 management schema 和相关表' AS step1,
       '已扩展 users 表，添加角色字段' AS step2,
       '已设置超级管理员账户' AS step3,
       '已创建示例租户数据' AS step4,
       '请使用以下账户登录：' AS next_step,
       'superadmin@example.com / admin123' AS superadmin_account;

