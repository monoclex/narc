use serenity::model::{
    channel::ReactionType,
    id::{GuildId, UserId},
};

use crate::database::Database;

impl Database {
    pub async fn protect_user(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<(), sqlx::Error> {
        let db_gid = guild_id.0 as i64;
        let db_uid = user_id.0 as i64;

        let mut transaction = self.connection.begin().await?;
        sqlx::query!(
            "
INSERT OR REPLACE INTO protected_users (guild_id, protected_user_id)
VALUES (?, ?)
            ",
            db_gid,
            db_uid,
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;

        Ok(())
    }

    pub async fn unprotect_user(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<(), sqlx::Error> {
        let db_gid = guild_id.0 as i64;
        let db_uid = user_id.0 as i64;

        let mut transaction = self.connection.begin().await?;
        sqlx::query!(
            "
DELETE FROM protected_users
WHERE guild_id = ?
  AND protected_user_id = ?
            ",
            db_gid,
            db_uid,
        )
        .execute(&mut transaction)
        .await?;
        transaction.commit().await?;

        Ok(())
    }
}
