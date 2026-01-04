
use axum::{http::Request, middleware::Next, response::Response};
use tracing::{info_span, Instrument};

pub async fn trace<B>(req: Request<B>, next: Next<B>) -> Response {
    let method = req.method().clone();
    let uri = req.uri().to_string();
    let span = info_span!("http", %method, %uri);
    next.run(req).instrument(span).await
}
