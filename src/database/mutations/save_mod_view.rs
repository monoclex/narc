use crate::database::{models::ModViewModel, Database};

impl Database {
    pub async fn save_mod_view(&self, view: ModViewModel) -> Result<(), sqlx::Error> {
        let db_rid = view.report_id as i64;
        let db_cid = view.channel_id.0 as i64;
        let db_mid = view.message_id.0 as i64;
        let db_paid = view.preview_archive_id as i64;
        let db_h = view.handler.map(|x| x.0 as i64);

        sqlx::query!(
            "
INSERT OR REPLACE INTO discord_mod_view (report_id, channel_id, message_id, preview_archive_id, handler)
VALUES (?, ?, ?, ?, ?)
            ",
            db_rid, db_cid, db_mid, db_paid, db_h
        )
        .execute(&self.connection)
        .await?;

        Ok(())
    }
}
