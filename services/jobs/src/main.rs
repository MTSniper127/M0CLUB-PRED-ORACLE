
use clap::Parser;
use tracing::info;

mod tasks;
mod scheduler;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "60")]
    interval_secs: u64,
}

fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let json = std::env::var("M0_LOG_JSON").ok().as_deref() == Some("1");
    if json {
        tracing_subscriber::fmt().with_env_filter(filter).json().with_target(true).init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).with_target(true).init();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let args = Args::parse();
    info!(interval_secs=args.interval_secs, "jobs starting");
    scheduler::run_loop(std::time::Duration::from_secs(args.interval_secs.max(5))).await?;
    Ok(())
}
