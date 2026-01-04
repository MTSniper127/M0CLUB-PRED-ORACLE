
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    info!("recompute task executed");
    Ok(())
}
