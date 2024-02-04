use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use chrono::{Datelike, Local};
use futures::executor;
use log::warn;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Serialize, Serializer};
use tera::Context;

use entity::{
    metadata::{self, Model as Metadata},
    prelude::Metadata as MetadataEntity,
};

use crate::{
    article_manager::ArticleManager,
    config::{Config, Locale},
    db::connect_to_db,
    text::Text,
};

enum Type {
    String(String),
}

// This struct tracks all site wide settings
pub struct Site {
    pub article_manager: ArticleManager,
    text: Text,
    db: DatabaseConnection,
    metadata: Mutex<HashMap<&'static str, Type>>,
}

pub enum Str {
    Owned(String),
    Static(&'static str),
}

impl Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Str::Owned(s) => serializer.serialize_str(s),
            Str::Static(s) => serializer.serialize_str(s),
        }
    }
}

impl Site {
    const METADATA_KEYS: &'static [&'static str] = &["author", "site_name"];

    pub async fn new(config: &Config) -> Result<Self> {
        let db = connect_to_db(&config.database_path).await?;

        let am = ArticleManager::new(db.clone())?;

        let text = match config.locale {
            Locale::En => Text::en(),
            Locale::Zh => Text::zh(),
        };

        // Initialize metadata
        let mut metadata = HashMap::new();
        for key in Self::METADATA_KEYS {
            if let Some(value) = get_value_from_db(&db, key).await? {
                metadata.insert(*key, value);
            }
        }

        Ok(Self {
            article_manager: am,
            text,
            db,
            metadata: Mutex::new(metadata),
        })
    }

    async fn refresh_metadata(&self) -> Result<()> {
        let mut metadata = HashMap::new();
        for key in Self::METADATA_KEYS {
            if let Some(value) = get_value_from_db(&self.db, key).await? {
                metadata.insert(*key, value);
            }
        }

        match self.metadata.lock() {
            Ok(mut m) => *m = metadata,
            Err(mut e) => {
                **e.get_mut() = metadata;
                self.metadata.clear_poison();
            }
        }

        Ok(())
    }

    pub fn get_author(&self) -> Str {
        self.get_metadata_string("author", self.text.author_default)
    }

    pub fn get_site_name(&self) -> Str {
        self.get_metadata_string("site_name", self.text.site_name_default)
    }

    pub fn format_time(&self, timestamp: i64) -> String {
        (self.text.format_time)(timestamp)
    }

    fn get_metadata_string(&self, key: &str, default: &'static str) -> Str {
        let lock = match self.metadata.lock() {
            Ok(lock) => lock,
            Err(_) => {
                warn!("Metadata lock poisoned, refreshing");
                if let Err(e) = executor::block_on(self.refresh_metadata()) {
                    warn!("Failed to refresh metadata: {e}. Is database down?");
                }

                return Str::Static(default);
            }
        };

        let value = match lock.get(key) {
            Some(Type::String(s)) => Str::Owned(s.to_owned()),
            None => Str::Static(default),
        };

        value
    }

    pub fn base_context(&self) -> Context {
        let mut context = Context::new();

        let year = Local::now().year().to_string();
        context.insert("year", &year);

        let site_name = self.get_site_name();
        context.insert("site_name", &site_name);

        context
    }
}

async fn get_value_from_db(db: &DatabaseConnection, key: &str) -> Result<Option<Type>> {
    let entry: Option<Metadata> = MetadataEntity::find()
        .filter(metadata::Column::Key.eq(key))
        .one(db)
        .await?
        .map(|e| e.into());

    let entry = match entry {
        Some(e) => e,
        None => return Ok(None),
    };

    let entry = match (entry.val_int, entry.val_float, entry.val_string) {
        (Some(_), _, _) => todo!(),
        (_, Some(_), _) => todo!(),
        (_, _, Some(s)) => Type::String(s),
        _ => return Err(anyhow::anyhow!("no value found for key {key}")),
    };

    Ok(Some(entry))
}
