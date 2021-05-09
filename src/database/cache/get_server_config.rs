use serenity::model::id::GuildId;

use crate::database::{Database, ServerConfiguration};

impl Database {
    pub async fn get_server_config(
        &self,
        guild_id: &GuildId,
    ) -> Result<Option<ServerConfiguration>, sqlx::Error> {
        {
            let lock = self.cache.server_configs.read().await;
            if let Some(prefix) = lock.get(guild_id) {
                return Ok(prefix.to_owned());
            }
        }

        let prefix = self.load_server_config(guild_id).await?;

        {
            let mut lock = self.cache.server_configs.write().await;
            lock.insert(guild_id.to_owned(), prefix.clone());
        }

        Ok(prefix)
    }
}
