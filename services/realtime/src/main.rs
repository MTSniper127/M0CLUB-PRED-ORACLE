
use axum::{routing::get, Router};
use clap::Parser;
use tracing::info;

mod ws;
mod pubsub;
mod throttling;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:8090")]
    bind: String,
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

    let hub = pubsub::Hub::new();
    // publish demo events
    let hub2 = hub.clone();
    tokio::spawn(async move {
        let mut i = 0u64;
        loop {
            i += 1;
            hub2.publish("predictions", format!("{{"type":"tick","i":{i}}}"));
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let state = ws::AppState { hub };
    let app = Router::new()
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    info!(bind=%args.bind, "realtime listening");
    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
