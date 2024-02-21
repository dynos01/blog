use anyhow::Result;
use chrono::{Datelike, Local, TimeZone};
use hashbrown::HashMap;
use log::{debug, warn};
use parking_lot::RwLock;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;

use entity::{
    article::{self, Model as Article},
    prelude::Article as ArticleEntity,
};

pub struct ArticleManager {
    db: DatabaseConnection,
    cache: RwLock<HashMap<String, Article>>,
}

#[derive(Serialize)]
pub struct ArticleMetadata {
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub created: String,
    pub updated: Option<String>,
    pub url: String,
    pub year: u64,
}

impl ArticleManager {
    pub fn new(db: DatabaseConnection) -> Result<Self> {
        let cache = RwLock::new(HashMap::new());
        Ok(Self { db, cache })
    }

    pub async fn get_article(&self, url: &str) -> Result<Option<Article>> {
        // First, check the cache
        if let Some(article) = self.cache.read().get(url) {
            debug!("Cache hit for article {url}");
            return Ok(Some(article.to_owned()));
        }

        debug!("Cache miss for article {url}");

        // Then, check the database
        let article: Option<Article> = ArticleEntity::find()
            .filter(article::Column::Url.eq(url))
            .one(&self.db)
            .await?
            .map(|article| article.into());

        // Update the cache
        if let Some(article) = &article {
            self.cache.write().insert(url.to_owned(), article.clone());
        }

        let article = match article {
            Some(article) => article,
            None => {
                debug!("No article found with url {url}");
                return Ok(None);
            }
        };

        debug!("Found article with url {url}");

        Ok(Some(article))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Article>> {
        let articles: Vec<Article> = ArticleEntity::find()
            .filter(
                article::Column::Title
                    .contains(query)
                    .or(article::Column::Content.contains(query)),
            )
            .all(&self.db)
            .await?
            .into_iter()
            .map(|article| article.into())
            .collect();

        debug!("Found {} article(s) matching query {query}", articles.len());

        Ok(articles)
    }

    pub fn _remove(&self, article: &str) {
        self.cache.write().remove(article);
    }

    pub async fn get_all_article_metadatas(&self) -> Result<Vec<ArticleMetadata>> {
        let mut articles: Vec<Article> = ArticleEntity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|article| article.into())
            .collect();

        articles.sort_by(|a, b| {
            b.updated
                .unwrap_or(b.created)
                .cmp(&a.updated.unwrap_or(a.created))
        });

        // sort by update time
        let mut articles = articles;
        articles.sort_by(|a, b| {
            b.updated
                .unwrap_or(b.created)
                .cmp(&a.updated.unwrap_or(a.created))
        });

        let metadata: Vec<ArticleMetadata> = articles
            .into_iter()
            .map(|article| {
                let year = {
                    let timestamp = article.updated.unwrap_or(article.created);
                    Local
                        .timestamp_opt(timestamp, 0)
                        .map(|dt| dt.year())
                        .single()
                        .unwrap_or_else(|| {
                            warn!("Invalid timestamp {timestamp}");
                            0
                        })
                } as u64;

                ArticleMetadata {
                    title: article.title,
                    excerpt: article.excerpt,
                    // TODO: real format
                    created: format!("{}", article.created),
                    updated: article.updated.map(|t| format!("{}", t)),
                    url: article.url,
                    year,
                }
            })
            .collect();

        Ok(metadata)
    }
}
