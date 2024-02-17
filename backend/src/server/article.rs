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

pub async fn article_handler(
    Extension(site): Extension<Arc<Site>>,
    Path(url): Path<String>,
) -> impl IntoResponse {
    let article = match article_impl(site, &url).await {
        Ok(Some(article)) => article,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!("Failed to handle request for {url}: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let headers = [(header::CONTENT_TYPE, "text/html")];

    (StatusCode::OK, headers, article).into_response()
}

async fn article_impl(site: Arc<Site>, url: &str) -> Result<Option<String>> {
    let article = site.article_manager.get_article(url).await?;
    let article = match article {
        Some(article) => article,
        None => return Ok(None),
    };

    let author = site.get_author();
    let title = article.title.unwrap_or(String::from("(no title)"));
    let content = article.content.unwrap_or(String::from(""));
    let created = site.format_time(article.created);

    let mut context = site.base_context();
    context.insert("title", &title);
    context.insert("author", &author);
    context.insert("content", &content);
    context.insert("created", &created);

    let rendered = site.render("article.html", &context)?;
    let minified = minify_html(rendered)?;

    Ok(Some(minified))
}
