use serenity::prelude::TypeMapKey;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use self::cache::Cache;

mod cache;
mod mutations;
pub use mutations::*;
mod reads;
pub use reads::*;
pub mod models;

#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ReportId(pub u64);

impl std::fmt::Display for ReportId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Database {
    pub connection: Pool<Sqlite>,
    pub cache: Cache,
}

impl TypeMapKey for Database {
    type Value = Self;
}

impl Database {
    pub async fn connect(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(num_cpus::get() as u32)
            .connect(connection_string)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Database {
            connection: pool,
            cache: Cache::new(),
        })
    }
}
