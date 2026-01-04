
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    info!("backfill task executed");
    Ok(())
}
