
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    info!("reconcile task executed");
    Ok(())
}
