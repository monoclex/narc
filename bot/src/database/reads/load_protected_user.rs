use serenity::model::id::{GuildId, UserId};

use crate::database::Database;

impl Database {
    pub async fn load_protected_user(
        &self,
        guild_id: &GuildId,
        protected_user_id: &UserId,
    ) -> Result<bool, sqlx::Error> {
        let db_gid = guild_id.0 as i64;
        let db_protected_user_id = protected_user_id.0 as i64;

        let result = sqlx::query!(
            "
SELECT * FROM protected_users
WHERE guild_id = ?
  AND protected_user_id = ?;
            ",
            db_gid,
            db_protected_user_id
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(result.is_some())
    }
}
