use serenity::model::id::{GuildId, UserId};

use crate::database::Database;

impl Database {
    pub async fn load_protected_users(
        &self,
        guild_id: &GuildId,
    ) -> Result<Vec<UserId>, sqlx::Error> {
        let db_gid = guild_id.0 as i64;

        let results = sqlx::query!(
            "
SELECT protected_user_id FROM protected_users
WHERE guild_id = ?;
            ",
            db_gid
        )
        .fetch_all(&self.connection)
        .await?;

        let users = results
            .into_iter()
            .map(|u| UserId(u.protected_user_id as u64))
            .collect::<Vec<_>>();

        Ok(users)
    }
}
