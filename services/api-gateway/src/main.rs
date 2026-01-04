
use axum::{routing::get, Router, middleware as axum_middleware};
use tower_http::cors::{CorsLayer, Any};
use tracing::info;
use clap::Parser;

mod routes;
mod openapi;
mod auth;
mod cache;
mod db;
mod middleware;
mod error;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,

    #[arg(long, default_value = "600")]
    rate_limit_per_minute: u32,

    #[arg(long, default_value = "false")]
    enable_openapi: bool,
}

fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,sqlx=warn"));
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

    let limiter = middleware::rate_limit::RateLimiter::new(args.rate_limit_per_minute);

    let mut app = Router::new()
        .route("/health", get(routes::health::get))
        .route("/markets", get(routes::markets::list))
        .route("/epochs", get(routes::epochs::list))
        .route("/predictions/:market_id/latest", get(routes::predictions::latest))
        .route("/metrics", get(routes::metrics::get))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(axum_middleware::from_fn(middleware::tracing::trace))
        .layer(axum_middleware::from_fn(middleware::request_id::request_id))
        .with_state(());

    // Attach extensions (RateLimiter) through a middleware closure.
    app = app.layer(axum_middleware::from_fn(move |req, next| {
        let limiter = limiter.clone();
        async move {
            let mut req = req;
            req.extensions_mut().insert(limiter);
            middleware::rate_limit::rate_limit(req, next).await
        }
    }));

    if args.enable_openapi {
        app = app.route("/openapi.json", get(|| async {
            axum::Json(openapi::spec::openapi_spec())
        }));
    }

    info!(bind=%args.bind, "api-gateway listening");
    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
