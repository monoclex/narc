use serenity::model::id::GuildId;

use crate::database::Database;

impl Database {
    pub async fn get_server_prefix(
        &self,
        guild_id: &GuildId,
    ) -> Result<Option<String>, sqlx::Error> {
        {
            let lock = self.cache.server_prefixes.read().await;
            if let Some(prefix) = lock.get(guild_id) {
                return Ok(prefix.to_owned());
            }
        }

        let prefix = self.load_server_prefix(guild_id).await?;

        {
            let mut lock = self.cache.server_prefixes.write().await;
            lock.insert(guild_id.to_owned(), prefix.clone());
        }

        Ok(prefix)
    }
}
