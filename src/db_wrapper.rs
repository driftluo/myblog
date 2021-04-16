use once_cell::sync::OnceCell;
use redis::aio::ConnectionManager;
use sqlx::postgres::PgPool;
use std::{env, fmt};

static REDIS: OnceCell<RedisManager> = OnceCell::new();
static POSTGRES: OnceCell<PgPool> = OnceCell::new();

pub struct RedisManager {
    pool: ConnectionManager,
    script: Option<redis::Script>,
}

impl RedisManager {
    pub async fn new<T>(address: T) -> Self
    where
        T: redis::IntoConnectionInfo,
    {
        let re = redis::Client::open(address).unwrap();
        let manager = re.get_tokio_connection_manager().await.unwrap();
        Self {
            pool: manager,
            script: None,
        }
    }

    pub async fn new_with_script<T>(address: T, path: &str) -> Self
    where
        T: redis::IntoConnectionInfo,
    {
        let re = redis::Client::open(address).unwrap();
        let manager = re.get_tokio_connection_manager().await.unwrap();
        let lua = tokio::fs::read_to_string(path).await.unwrap();

        Self {
            pool: manager,
            script: Some(redis::Script::new(&lua)),
        }
    }

    pub async fn keys(&self, pattern: &str) -> Vec<String> {
        redis::cmd("keys")
            .arg(pattern)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn exists(&self, redis_key: &str) -> bool {
        redis::cmd("exists")
            .arg(redis_key)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn expire(&self, redis_key: &str, sec: i64) {
        redis::cmd("expire")
            .arg(redis_key)
            .arg(sec)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn del<T>(&self, redis_keys: T) -> bool
    where
        T: redis::ToRedisArgs,
    {
        redis::cmd("del")
            .arg(redis_keys)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn set(&self, redis_key: &str, value: &str) {
        redis::cmd("set")
            .arg(redis_key)
            .arg(value)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn get(&self, redis_key: &str) -> Result<String, redis::RedisError> {
        redis::cmd("get")
            .arg(redis_key)
            .query_async(&mut self.pool.clone())
            .await
    }

    pub async fn hset<T>(&self, redis_key: &str, hash_key: &str, value: T)
    where
        T: redis::ToRedisArgs,
    {
        redis::cmd("hset")
            .arg(redis_key)
            .arg(hash_key)
            .arg(value)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn hdel<T>(&self, redis_key: &str, hash_key: T)
    where
        T: redis::ToRedisArgs,
    {
        redis::cmd("hdel")
            .arg(redis_key)
            .arg(hash_key)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn hget<T>(&self, redis_key: &str, hash_key: &str) -> Result<T, redis::RedisError>
    where
        T: redis::FromRedisValue,
    {
        redis::cmd("hget")
            .arg(redis_key)
            .arg(hash_key)
            .query_async(&mut self.pool.clone())
            .await
    }

    pub async fn hexists(&self, redis_key: &str, hash_key: &str) -> bool {
        redis::cmd("hexists")
            .arg(redis_key)
            .arg(hash_key)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn lpush<T>(&self, redis_key: &str, value: T)
    where
        T: redis::ToRedisArgs,
    {
        redis::cmd("lpush")
            .arg(redis_key)
            .arg(value)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn llen<T>(&self, redis_key: &str) -> T
    where
        T: redis::FromRedisValue,
    {
        redis::cmd("llen")
            .arg(redis_key)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn ltrim(&self, redis_key: &str, start: i64, stop: i64) {
        redis::cmd("ltrim")
            .arg(redis_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn lrem<T>(&self, redis_key: &str, count: i64, value: T)
    where
        T: redis::ToRedisArgs,
    {
        redis::cmd("lrem")
            .arg(redis_key)
            .arg(count)
            .arg(value)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn lrange<T>(&self, redis_key: &str, start: i64, stop: i64) -> T
    where
        T: redis::FromRedisValue,
    {
        redis::cmd("lrange")
            .arg(redis_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut self.pool.clone())
            .await
            .unwrap()
    }

    pub async fn lua_push(&self, redis_key: &str, ip: &str) {
        if let Some(lua) = self.script.as_ref() {
            lua.arg(redis_key)
                .arg(ip)
                .invoke_async(&mut self.pool.clone())
                .await
                .unwrap()
        }
    }
}

impl fmt::Debug for RedisManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisManager")
            .field("script", &self.script)
            .finish()
    }
}

pub async fn create_redis_pool(path: Option<&str>) {
    let database_url = env::var("REDIS_URL").expect("DATABASE_URL must be set");
    let pool = match path {
        Some(path) => RedisManager::new_with_script(database_url.as_str(), path).await,
        None => RedisManager::new(database_url.as_str()).await,
    };

    REDIS.set(pool).expect("Redis global pool must set success");
}

pub async fn create_pg_pool() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool.");
    POSTGRES
        .set(pool)
        .expect("Postgresql global pool must set success")
}

#[inline]
pub fn get_postgres() -> &'static PgPool {
    // Safety: tt is already set when the program is initialized
    unsafe { POSTGRES.get_unchecked() }
}

#[inline]
pub fn get_redis() -> &'static RedisManager {
    // Safety: tt is already set when the program is initialized
    unsafe { REDIS.get_unchecked() }
}
