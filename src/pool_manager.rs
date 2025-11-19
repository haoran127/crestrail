use dashmap::DashMap;
use once_cell::sync::Lazy;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::error::{AppError, Result};

/// 全局连接池管理器
pub static POOL_MANAGER: Lazy<PoolManager> = Lazy::new(|| PoolManager::new());

/// 数据库连接配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub id: i32,
    pub host: String,
    pub port: i32,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

impl DatabaseConfig {
    /// 构建连接 URL
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// 连接池管理器
pub struct PoolManager {
    pools: DashMap<i32, PgPool>,
}

impl PoolManager {
    /// 创建新的连接池管理器
    pub fn new() -> Self {
        Self {
            pools: DashMap::new(),
        }
    }

    /// 获取或创建连接池
    pub async fn get_or_create_pool(&self, config: DatabaseConfig) -> Result<PgPool> {
        // 如果连接池已存在，直接返回
        if let Some(pool) = self.pools.get(&config.id) {
            // 验证连接池是否健康
            if pool.acquire().await.is_ok() {
                return Ok(pool.clone());
            } else {
                // 连接池不健康，移除它
                tracing::warn!("连接池 {} 不健康，将重新创建", config.id);
                self.pools.remove(&config.id);
            }
        }

        // 创建新的连接池
        tracing::info!(
            "创建新连接池: database_id={}, host={}:{}, database={}",
            config.id,
            config.host,
            config.port,
            config.database
        );

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connection_timeout))
            .connect(&config.connection_url())
            .await
            .map_err(|e| {
                tracing::error!("创建连接池失败: {}", e);
                AppError::Internal(format!("连接数据库失败: {}", e))
            })?;

        // 缓存连接池
        self.pools.insert(config.id, pool.clone());

        Ok(pool)
    }

    /// 移除连接池
    pub async fn remove_pool(&self, database_id: i32) {
        if let Some((_, pool)) = self.pools.remove(&database_id) {
            tracing::info!("关闭连接池: database_id={}", database_id);
            pool.close().await;
        }
    }

    /// 获取所有活跃的连接池数量
    pub fn active_pools_count(&self) -> usize {
        self.pools.len()
    }

    /// 清理所有连接池
    pub async fn clear_all(&self) {
        tracing::info!("关闭所有连接池");
        let ids: Vec<i32> = self.pools.iter().map(|entry| *entry.key()).collect();
        for id in ids {
            self.remove_pool(id).await;
        }
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

