use serenity::{
    model::{
        channel::ReactionType,
        id::{ChannelId, GuildId},
    },
    prelude::TypeMapKey,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::instrument;

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

impl TypeMapKey for Database {
    type Value = Self;
}

impl Database {
    pub async fn configure_server(
        &self,
        guild_id: GuildId,
        report_emoji: ReactionType,
        reports_channel: ChannelId,
        prefix: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let db_gid = guild_id.0 as i64;

        let reports_channel = reports_channel.0 as i64;

        let (emoji_builtin, emoji_custom) = match report_emoji {
            ReactionType::Custom { id, .. } => (None, Some(id.0 as i64)),
            ReactionType::Unicode(str) => (Some(str), None),
            _ => panic!("unsupported emoji type"),
        };

        let mut transaction = self.connection.begin().await?;
        sqlx::query!(
            "
INSERT OR REPLACE INTO server_configuration (guild_id, reports_channel, emoji_builtin, emoji_custom, prefix)
VALUES (?, ?, ?, ?, ?)
            ",
            db_gid,
            reports_channel,
            emoji_builtin,
            emoji_custom,
            prefix
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;

        self.cache.wipe_server_config_cache(&guild_id).await;

        Ok(())
    }
}

#[derive(Clone)]
pub struct ServerConfiguration {
    // pub guild_id: u64,
    pub emoji_builtin: Option<String>,
    pub emoji_custom: Option<u64>,
    pub prefix: Option<String>,
    pub reports_channel: u64,
}

impl ServerConfiguration {
    pub fn matches_emoji(&self, emoji: &ReactionType) -> bool {
        match (self.emoji_builtin.as_ref(), self.emoji_custom, emoji) {
            (_, Some(custom), ReactionType::Custom { id, .. }) => custom == id.0,
            (Some(builtin), _, ReactionType::Unicode(unicode)) => builtin == unicode,
            (None, None, ReactionType::Unicode(unicode)) => unicode == "🚩",
            _ => false,
        }
    }
}

pub struct Database {
    pub connection: Pool<Sqlite>,
    pub cache: Cache,
}

impl Database {
    #[instrument]
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
