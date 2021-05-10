use serenity::model::id::GuildId;
use sqlx::types::chrono::{DateTime, Utc};

use crate::database::Database;

impl Database {
    pub async fn load_welcome_time(
        &self,
        guild_id: &GuildId,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        let db_gid = guild_id.0 as i64;

        let time = sqlx::query!(
            "
SELECT welcomed FROM welcomed_servers
WHERE guild_id = ?;
            ",
            db_gid
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(time.map(|time| DateTime::<Utc>::from_utc(time.welcomed, Utc)))
    }
}
