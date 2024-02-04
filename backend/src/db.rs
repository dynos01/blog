use std::path::Path;

use anyhow::Result;
use log::debug;
use sea_orm::{Database, DatabaseConnection};
use tokio::fs::File;

use migration::{Migrator, MigratorTrait};

pub async fn connect_to_db(path: &str) -> Result<DatabaseConnection> {
    if !Path::new(path).exists() {
        File::create(path).await?;
    }

    let url = format!("sqlite://{}", path);
    let db = Database::connect(url).await?;

    Migrator::up(&db, None).await?;

    debug!("Opened database {path}");

    Ok(db)
}
