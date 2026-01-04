
use axum::{http::Request, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn request_id<B>(mut req: Request<B>, next: Next<B>) -> Response {
    let rid = Uuid::new_v4().to_string();
    req.headers_mut().insert("x-request-id", rid.parse().unwrap());
    let mut res = next.run(req).await;
    res.headers_mut().insert("x-request-id", rid.parse().unwrap());
    res
}
