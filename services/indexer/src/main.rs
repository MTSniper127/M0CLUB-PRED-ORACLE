
use clap::Parser;
use tokio::sync::mpsc;
use tracing::{info, warn};

mod program_subscriber;
mod parser;
mod storage;
mod reorg_handling;
mod error;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,

    #[arg(long)]
    postgres_url: Option<String>,

    #[arg(long)]
    clickhouse_url: Option<String>,
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
    info!(rpc=%args.rpc_url, "indexer starting");

    let (tx, mut rx) = mpsc::channel(256);
    let sub = program_subscriber::Subscriber::new(&args.rpc_url);
    tokio::spawn(async move {
        let _ = sub.poll_logs(tx).await;
    });

    let pg = if let Some(url) = &args.postgres_url {
        Some(storage::postgres::PgStore::connect(url).await?)
    } else { None };

    let ch = args.clickhouse_url.as_ref().map(|u| storage::clickhouse::ChStore::new(u));

    while let Some(resp) = rx.recv().await {
        for line in resp.logs {
            if let Some(ev) = parser::m0_oracle::parse_oracle_log(0, &line) {
                let json = serde_json::to_string(&ev)?;
                if let Some(pg) = &pg { pg.write_line(ev.slot, "oracle", &json).await.ok(); }
                if let Some(ch) = &ch { ch.write_line(ev.slot, "oracle", &json).await.ok(); }
                info!(kind="oracle", "parsed");
                continue;
            }
            if let Some(ev) = parser::m0_registry::parse_registry_log(0, &line) {
                let json = serde_json::to_string(&ev)?;
                if let Some(pg) = &pg { pg.write_line(ev.slot, "registry", &json).await.ok(); }
                if let Some(ch) = &ch { ch.write_line(ev.slot, "registry", &json).await.ok(); }
                info!(kind="registry", "parsed");
                continue;
            }
            warn!(line=%line, "unrecognized log line");
        }
    }

    Ok(())
}
