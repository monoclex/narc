use serenity::model::id::GuildId;

use crate::database::Database;

impl Database {
    pub async fn delete_welcome(&self, server: &GuildId) -> Result<(), sqlx::Error> {
        let db_gid = server.0 as i64;

        sqlx::query!(
            "
DELETE FROM welcomed_servers
WHERE guild_id = ?;
            ",
            db_gid
        )
        .execute(&self.connection)
        .await?;

        Ok(())
    }
}
