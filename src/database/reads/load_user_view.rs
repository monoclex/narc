use serenity::model::id::*;

use crate::database::{models::*, Database};

impl Database {
    pub async fn load_user_view(
        &self,
        report_id: u64,
    ) -> Result<Option<UserViewModel>, sqlx::Error> {
        let db_id = report_id as i64;

        let report = sqlx::query!(
            "
 SELECT * FROM discord_user_view WHERE report_id = ?
             ",
            db_id
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(report.map(|r| UserViewModel {
            report_id: r.report_id as u64,
            message_id: MessageId(r.message_id as u64),
            status: r.status.into(),
        }))
    }
}
