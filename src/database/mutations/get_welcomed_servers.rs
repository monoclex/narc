use std::collections::HashSet;

use crate::database::Database;

impl Database {
    pub async fn get_welcomed_servers(&self) -> Result<HashSet<u64>, sqlx::Error> {
        let guilds = sqlx::query!(
            "
SELECT guild_id FROM welcomed_servers;
            "
        )
        .fetch_all(&self.connection)
        .await?;

        Ok(guilds
            .iter()
            .map(|record| record.guild_id as u64)
            .collect::<HashSet<_>>())
    }
}
