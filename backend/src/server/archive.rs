use std::sync::Arc;

use anyhow::Result;
use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Extension,
};
use log::warn;
use serde::Serialize;

use crate::{site::Site, util::*};

pub async fn archive_handler(Extension(site): Extension<Arc<Site>>) -> impl IntoResponse {
    let text = match archive_impl(site).await {
        Ok(text) => text,
        Err(e) => {
            warn!("Failed to handle request for /archive: {e}");
            // TODO: shouldn't use bad request only here
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let headers = [(header::CONTENT_TYPE, "text/html")];

    (StatusCode::OK, headers, text).into_response()
}

async fn archive_impl(site: Arc<Site>) -> Result<String> {
    // This object should be as simple as possible to increase serialization speed
    #[derive(Serialize)]
    struct Article {
        title: String,
        url: String,
    }

    #[derive(Serialize)]
    struct Year {
        year: u64,
        articles: Vec<Article>,
    }

    #[derive(Serialize)]
    struct Archive {
        years: Vec<Year>,
    }

    // TODO: could be optimized
    let metadata: Vec<_> = site
        .article_manager
        .get_all_article_metadatas()
        .await?
        .into_iter()
        .collect();

    let mut years = vec![];
    let mut curr = Year {
        year: 0,
        articles: vec![],
    };

    for article in metadata {
        let year = article.year;
        let title = article.title.unwrap(); // todo: clean db
        let url = article.url;

        if year == curr.year {
            curr.articles.push(Article { title, url });
        } else {
            if !curr.articles.is_empty() {
                years.push(curr);
            }

            curr = Year {
                year,
                articles: vec![Article { title, url }],
            };
        }
    }

    if !curr.articles.is_empty() {
        years.push(curr);
    }

    let archive = Archive { years };

    let mut context = site.base_context();
    context.insert("archive", &archive);
    context.insert("title", "Archive"); // TODO: i18n

    let rendered = site.render("archive.html", &context)?;
    let minified = minify_html(rendered)?;

    Ok(minified)
}
