use std::sync::Arc;

use anyhow::Result;
use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Extension,
};
use log::warn;

use crate::{site::Site, util::*};

pub async fn editor_handler(Extension(site): Extension<Arc<Site>>) -> impl IntoResponse {
    let text = match editor_impl(site).await {
        Ok(text) => text,
        Err(e) => {
            warn!("Failed to handle request for /editor: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let headers = [(header::CONTENT_TYPE, "text/html")];

    (StatusCode::OK, headers, text).into_response()
}

async fn editor_impl(site: Arc<Site>) -> Result<String> {
    let mut context = site.base_context();
    context.insert("title", "Editor"); // TODO: i18n

    let rendered = site.render("editor.html", &context)?;
    let minified = minify_html(rendered)?;

    Ok(minified)
}
