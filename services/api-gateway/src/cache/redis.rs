
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        Ok(Self { client: redis::Client::open(url)? })
    }

    pub async fn get_json(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let v: Option<String> = conn.get(key).await?;
        Ok(v)
    }

    pub async fn set_json_ex(&self, key: &str, json: &str, ttl_secs: usize) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key, json, ttl_secs).await?;
        Ok(())
    }
}
