use serenity::model::id::GuildId;

use crate::database::Database;

impl Database {
    pub async fn make_welcome(&self, server: &GuildId) -> Result<(), sqlx::Error> {
        let db_gid = server.0 as i64;

        sqlx::query!(
            r#"
INSERT OR REPLACE INTO welcomed_servers (guild_id, welcomed)
VALUES (?, DATETIME("now"));
            "#,
            db_gid
        )
        .execute(&self.connection)
        .await?;

        Ok(())
    }
}
