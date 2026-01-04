
use axum::{http::Request, middleware::Next, response::Response};
use std::time::{Duration, Instant};
use dashmap::DashMap;

#[derive(Clone)]
pub struct RateLimiter {
    // key -> (count, window_start)
    map: std::sync::Arc<DashMap<String, (u32, Instant)>>,
    pub max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self { map: std::sync::Arc::new(DashMap::new()), max_per_minute }
    }
}

pub async fn rate_limit<B>(req: Request<B>, next: Next<B>) -> Response {
    // This middleware uses an in-process limiter keyed by client IP header (x-forwarded-for) or "local".
    // For distributed deployments, use Redis/Envoy instead.
    let limiter: RateLimiter = req.extensions().get::<RateLimiter>().cloned().unwrap_or_else(|| RateLimiter::new(600));
    let key = req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("local")
        .split(',')
        .next()
        .unwrap_or("local")
        .trim()
        .to_string();

    let now = Instant::now();
    let window = Duration::from_secs(60);
    let mut allow = true;

    limiter.map.alter(&key, |entry| {
        let (mut count, mut start) = entry.unwrap_or((0, now));
        if now.duration_since(start) > window {
            count = 0;
            start = now;
        }
        count += 1;
        if count > limiter.max_per_minute {
            allow = false;
        }
        Some((count, start))
    });

    if !allow {
        return axum::http::StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    next.run(req).await
}
