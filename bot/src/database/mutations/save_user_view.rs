use crate::database::{models::UserViewModel, Database};

impl Database {
    pub async fn save_user_view(&self, view: UserViewModel) -> Result<(), sqlx::Error> {
        let db_rid = view.report_id as i64;
        let db_mid = view.message_id.0 as i64;
        let db_s: i64 = view.status.into();

        sqlx::query!(
            "
INSERT OR REPLACE INTO discord_user_view (report_id, message_id, status)
VALUES (?, ?, ?)
            ",
            db_rid,
            db_mid,
            db_s
        )
        .execute(&self.connection)
        .await?;

        Ok(())
    }
}
