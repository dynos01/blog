use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
    Extension,
};
use log::warn;

use crate::{site::Site, util::*};

const ARTICLES_PER_PAGE: usize = 10;

pub async fn index_handler(Extension(site): Extension<Arc<Site>>) -> impl IntoResponse {
    let text = match index_impl(site, 0).await {
        Ok(text) => text,
        Err(e) => {
            warn!("Failed to handle request for /: {e}");
            // TODO: shouldn't use bad request only here
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let headers = [(header::CONTENT_TYPE, "text/html")];

    (StatusCode::OK, headers, text).into_response()
}

pub async fn index_page_handler(
    Extension(site): Extension<Arc<Site>>,
    Path(page): Path<usize>,
) -> impl IntoResponse {
    let text = match index_impl(site, page).await {
        Ok(text) => text,
        Err(e) => {
            warn!("Failed to handle request for /: {e}");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let headers = [(header::CONTENT_TYPE, "text/html")];

    (StatusCode::OK, headers, text).into_response()
}

async fn index_impl(site: Arc<Site>, page: usize) -> Result<String> {
    // TODO: could be optimized
    let metadata: Vec<_> = site
        .article_manager
        .get_all_article_metadatas()
        .await?
        .into_iter()
        .collect();

    let page = page - 1; // 1-indexed to 0-indexed

    let has_next = metadata.len() > (page + 1) * ARTICLES_PER_PAGE;
    let has_prev = page > 0;

    let metadata: Vec<_> = metadata
        .into_iter()
        .skip(page * ARTICLES_PER_PAGE)
        .take(ARTICLES_PER_PAGE)
        .collect();

    let mut context = site.base_context();
    context.insert("articles", &metadata);
    context.insert("title", "Home"); // TODO: i18n

    if has_next {
        // 0-indexed to 1-indexed
        // example: we are on page 1 (in url), 0-indexed is 0, so we want to show 2
        let next = page + 2;
        context.insert("next", &next);
    }

    if has_prev {
        let prev = page;
        context.insert("prev", &prev);
    }

    let rendered = site.render("index.html", &context)?;
    let minified = minify_html(rendered)?;

    Ok(minified)
}
