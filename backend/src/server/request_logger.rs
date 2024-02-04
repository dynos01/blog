use axum::{body::Body, http::Request, middleware::Next, response::IntoResponse};
use log::debug;

pub async fn request_logger(request: Request<Body>, next: Next) -> impl IntoResponse {
    let url = request.uri().path();
    debug!("Received request for {url}");

    next.run(request).await
}
