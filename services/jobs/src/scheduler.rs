
use std::time::Duration;
use tracing::info;

use crate::tasks;

pub async fn run_loop(interval: Duration) -> anyhow::Result<()> {
    loop {
        // Run a small fixed set of jobs.
        tasks::cleanup::run().await?;
        tasks::reconcile::run().await?;
        tasks::recompute::run().await?;
        tasks::backfill::run().await?;

        info!("jobs cycle complete");
        tokio::time::sleep(interval).await;
    }
}
