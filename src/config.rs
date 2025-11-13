use std::env;

/// 应用配置
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL 必须设置在环境变量中");

        let host = env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT 必须是有效的数字");

        Ok(Config {
            database_url,
            host,
            port,
        })
    }
}

