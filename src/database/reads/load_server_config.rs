use serenity::model::id::GuildId;

use crate::database::{Database, ServerConfiguration};

impl Database {
    pub async fn load_server_config(
        &self,
        guild_id: &GuildId,
    ) -> Result<Option<ServerConfiguration>, sqlx::Error> {
        let guild_id = guild_id.0 as i64;
        let server = sqlx::query!(
            "
SELECT * FROM server_configuration
WHERE guild_id = ?;
            ",
            guild_id
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(server.map(|server| ServerConfiguration {
            emoji_builtin: server.emoji_builtin,
            emoji_custom: server.emoji_custom.map(|n| n as u64),
            prefix: server.prefix,
            reports_channel: server.reports_channel as u64,
        }))
    }
}
