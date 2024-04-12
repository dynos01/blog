use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Mutex,
    task::{Context, Poll},
};

use anyhow::Result;
use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{HeaderValue, StatusCode},
    response::Response,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::future::BoxFuture;
use log::info;
use once_cell::sync::Lazy;
use tower::{Layer, Service};

use crate::util::*;

const MAX_RETRY: u64 = 5;
const COOLDOWN: i64 = 60 * 60 * 12; // 12 hours

static BLACKLIST: Lazy<Mutex<HashMap<IpAddr, BlacklistStatus>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

enum BlacklistStatus {
    Retry(u64),
    Bannded(i64),
}

#[derive(Clone)]
pub struct Auth {
    auth_encoded: String,
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    auth_encoded: String,
}

impl Default for BlacklistStatus {
    fn default() -> Self {
        Self::Retry(1)
    }
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

    fn auth(&self, header: &HeaderValue, ip: &IpAddr) -> bool {
        self.auth_impl(header, ip).unwrap_or_else(|e| {
            info!("Error in validating auth request: {e}, raw header: {header:?}");
            false
        })
    }

    fn auth_impl(&self, header: &HeaderValue, ip: &IpAddr) -> Result<bool> {
        const METHOD: &'static str = "Basic ";
        let header = header.to_str()?;

        if !header.starts_with(METHOD) {
            // An invalid request, no risk
            return Ok(false);
        }

        let mut lock = BLACKLIST.lock().unwrap();

        if let Some(status) = lock.get(&ip) {
            match status {
                BlacklistStatus::Retry(count) => {
                    if count >= &MAX_RETRY {
                        return Ok(false);
                    }
                }
                BlacklistStatus::Bannded(prev_timestamp) => {
                    if *prev_timestamp + COOLDOWN < timestamp() {
                        lock.remove(&ip);
                    } else {
                        return Ok(false);
                    }
                }
            }
        }

        drop(lock);

        let encoded = &header[METHOD.len()..];

        if encoded != self.auth_encoded {
            let mut lock = BLACKLIST.lock().unwrap();

            let new_state = match lock.get(&ip) {
                Some(old_state) => match old_state {
                    BlacklistStatus::Retry(count) => {
                        if *count != MAX_RETRY - 1 {
                            BlacklistStatus::Retry(count + 1)
                        } else {
                            info!("Blacklisting {ip} for {MAX_RETRY} failed auth attempt");
                            BlacklistStatus::Bannded(timestamp())
                        }
                    }
                    BlacklistStatus::Bannded(_) => BlacklistStatus::Bannded(timestamp()),
                },
                None => BlacklistStatus::default(),
            };

            lock.insert(*ip, new_state);

            return Ok(false);
        }

        let mut lock = BLACKLIST.lock().unwrap();
        lock.remove(&ip);

        Ok(true)
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

        let conn: &ConnectInfo<SocketAddr> = request.extensions().get().unwrap();
        let ip = &conn.0.ip();

        if !self.auth(auth_header, ip) {
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
