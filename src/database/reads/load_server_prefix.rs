use serenity::model::id::GuildId;

use crate::database::Database;

impl Database {
    pub async fn load_server_prefix(
        &self,
        guild_id: &GuildId,
    ) -> Result<Option<String>, sqlx::Error> {
        let db_gid = guild_id.0 as i64;

        let find_prefix = sqlx::query!(
            "
SELECT prefix FROM server_configuration
WHERE guild_id = ?
            ",
            db_gid
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(find_prefix.and_then(|r| r.prefix))
    }
}
