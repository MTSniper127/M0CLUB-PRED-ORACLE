
#[derive(Clone)]
pub struct ChStore {
    pub url: String,
}

impl ChStore {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string() }
    }

    pub async fn write_line(&self, _slot: u64, _kind: &str, _json: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
