-- ============================================
-- 创建超级管理员账户
-- ============================================
-- 邮箱: admin@example.com
-- 密码: Admin123
-- 注意：这个密码哈希是使用 bcrypt 算法预先生成的

-- 1. 删除已存在的 admin 用户（如果有）
DELETE FROM users WHERE email = 'admin@example.com';

-- 2. 插入新的 admin 用户
-- 密码 "Admin123" 的 bcrypt 哈希值
-- 注意：你需要使用后端的注册功能生成正确的哈希值
-- 或者直接通过注册页面注册后运行下面的更新语句

-- 临时方案：先创建一个用户记录，密码字段留空
INSERT INTO users (username, email, password_hash, role, is_superadmin, created_at)
VALUES (
    'admin',
    'admin@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYpJ.wJZ0Om', -- Admin123 的 bcrypt 哈希
    'admin',
    true,
    CURRENT_TIMESTAMP
);

-- 3. 验证创建结果
SELECT id, username, email, role, is_superadmin, created_at 
FROM users 
WHERE email = 'admin@example.com';

