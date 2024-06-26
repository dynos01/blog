use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{extract::Extension, middleware, routing::get, Router};
use log::debug;
use tokio::net::TcpListener;

use crate::{
    server::{
        archive::archive_handler,
        article::article_handler,
        asset::asset_handler,
        auth::Auth,
        editor::editor_handler,
        index::{index_handler, index_page_handler},
        request_logger::request_logger,
        search::search_handler,
    },
    site::Site,
};

pub async fn serve(bind: SocketAddr, site: Site) -> Result<()> {
    let editor = Router::new()
        .route("/editor", get(editor_handler))
        .layer(Auth::new(&site.admin.0, &site.admin.1));

    let app = Router::new()
        .merge(editor)
        .route("/", get(index_handler))
        .route("/archive", get(archive_handler))
        .route("/page/:page", get(index_page_handler))
        .route("/api/search", get(search_handler))
        .route("/article/:url", get(article_handler))
        .route("/asset/*path", get(asset_handler))
        .layer(middleware::from_fn(request_logger))
        .layer(Extension(Arc::new(site)));

    debug!("Listening on {bind}");
    let socket = TcpListener::bind(bind).await?;
    axum::serve(
        socket,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
