use serenity::model::id::GuildId;

use crate::database::Database;

impl Database {
    pub async fn get_server_prefix(
        &self,
        guild_id: &GuildId,
    ) -> Result<Option<String>, sqlx::Error> {
        Ok(self
            .get_server_config(guild_id)
            .await?
            .and_then(|c| c.prefix))
    }
}
