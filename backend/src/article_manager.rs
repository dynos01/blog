use anyhow::Result;
use hashbrown::HashMap;
use log::debug;
use parking_lot::RwLock;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tera::{Context, Tera};

use entity::{
    article::{self, Model as Article},
    prelude::Article as ArticleEntity,
};

pub struct ArticleManager {
    db: DatabaseConnection,
    cache: RwLock<HashMap<String, Article>>,
    templates: Tera,
}

impl ArticleManager {
    pub fn new(db: DatabaseConnection) -> Result<Self> {
        let cache = RwLock::new(HashMap::new());
        let templates = init_tera()?;
        Ok(Self {
            db,
            cache,
            templates,
        })
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

    pub fn render(&self, context: &Context) -> Result<String> {
        Ok(self.templates.render("article.html", context)?)
    }
}

fn init_tera() -> Result<Tera> {
    let templates = {
        mod template {
            include!(concat!(env!("OUT_DIR"), "/template.rs"));
        }

        template::TEMPLATES
            .into_iter()
            .map(|(name, template)| try {
                let template = std::str::from_utf8(template)?;
                (name, template)
            })
            .collect::<Result<Vec<_>>>()?
    };

    let mut tera = Tera::default();
    tera.add_raw_templates(templates)?;
    tera.autoescape_on(vec![]);

    debug!("Loaded {} template(s)", tera.templates.len());

    Ok(tera)
}
