use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{extract::Extension, middleware, routing::get, Router};
use log::debug;
use tokio::net::TcpListener;

use crate::{
    server::{
        article::article_handler, asset::asset_handler, request_logger::request_logger,
        search::search_handler,
    },
    site::Site,
};

pub async fn serve(bind: SocketAddr, site: Site) -> Result<()> {
    let app = Router::new()
        .route("/api/search", get(search_handler))
        .route("/article/:url", get(article_handler))
        .route("/asset/*path", get(asset_handler))
        .layer(middleware::from_fn(request_logger))
        .layer(Extension(Arc::new(site)));

    debug!("Listening on {bind}");
    let socket = TcpListener::bind(bind).await?;
    axum::serve(socket, app).await?;

    Ok(())
}
