
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct PgStore {
    pub pool: PgPool,
}

impl PgStore {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new().max_connections(10).connect(url).await?;
        Ok(Self { pool })
    }

    pub async fn write_line(&self, _slot: u64, _kind: &str, _json: &str) -> anyhow::Result<()> {
        // Skeleton: in production create tables and insert.
        Ok(())
    }
}
