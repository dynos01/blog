use std::task::{Context, Poll};

use anyhow::Result;
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::Response,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::future::BoxFuture;
use log::info;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct Auth {
    auth_encoded: String,
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    auth_encoded: String,
}

impl Auth {
    pub fn new(username: &str, password: &str) -> Self {
        let auth_decoded = format!("{}:{}", username, password);
        let auth_encoded = STANDARD.encode(auth_decoded.as_bytes());

        Self { auth_encoded }
    }
}

impl<S> Layer<S> for Auth {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware::new(inner, &self.auth_encoded)
    }
}

impl<S> AuthMiddleware<S> {
    fn new(inner: S, auth_encoded: &str) -> Self {
        Self {
            inner,
            auth_encoded: auth_encoded.to_owned(),
        }
    }

    fn auth(&self, header: &HeaderValue) -> bool {
        self.auth_impl(header).unwrap_or_else(|e| {
            info!("Error in validating auth request: {e}, raw header: {header:?}");
            false
        })
    }

    fn auth_impl(&self, header: &HeaderValue) -> Result<bool> {
        const METHOD: &'static str = "Basic ";
        let header = header.to_str()?;

        if !header.starts_with(METHOD) {
            return Ok(false);
        }

        let encoded = &header[METHOD.len()..];

        Ok(encoded == self.auth_encoded)
    }
}

impl<S> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let url = request.uri().path();

        let auth_header = match request.headers().get("Authorization") {
            Some(ah) => ah,
            None => {
                let resp = unauthorized(url);
                return Box::pin(async move { Ok(resp) });
            }
        };

        if !self.auth(auth_header) {
            let resp = unauthorized(url);
            return Box::pin(async move { Ok(resp) });
        }

        let future = self.inner.call(request);

        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}

fn unauthorized(url: &str) -> Response<Body> {
    info!("Unauthorized request to {url}");

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic")
        .body(Body::empty())
        .unwrap()
}
