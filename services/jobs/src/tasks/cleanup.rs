
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    info!("cleanup task executed");
    Ok(())
}
