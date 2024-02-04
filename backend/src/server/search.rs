mod proto {
    include!(concat!(env!("OUT_DIR"), "/blog.search.rs"));
}

use std::sync::Arc;

use anyhow::Result;
use axum::{body::Bytes, http::StatusCode, response::IntoResponse, Extension};
use log::warn;

use crate::{site::Site, util::*};

use self::proto::{SearchRequest, SearchResponse};

pub async fn search_handler(
    Extension(site): Extension<Arc<Site>>,
    body: Bytes,
) -> impl IntoResponse {
    let response = match search_impl(site, body.to_vec()).await {
        Ok(response) => response,
        Err(e) => {
            warn!("Failed to handle request: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let headers = [("Content-Type", "application/octet-stream")];
    (StatusCode::OK, headers, response).into_response()
}

async fn search_impl(site: Arc<Site>, body: Vec<u8>) -> Result<Vec<u8>> {
    let request: SearchRequest = protobuf_decode(&body)?;
    let result: Vec<_> = site
        .article_manager
        .search(&request.query)
        .await?
        .into_iter()
        .filter_map(|a| a.title)
        .collect();
    let response = SearchResponse { result };
    let response = protobuf_encode(response)?;
    Ok(response)
}
