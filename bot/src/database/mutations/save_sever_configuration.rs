use serenity::model::{
    channel::ReactionType,
    id::{ChannelId, GuildId},
};

use crate::database::Database;

impl Database {
    pub async fn save_server_configuration(
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
