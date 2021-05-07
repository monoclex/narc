use serenity::model::id::*;

use crate::database::{models::*, Database};

impl Database {
    pub async fn load_mod_view(&self, report_id: u64) -> Result<Option<ModViewModel>, sqlx::Error> {
        let db_id = report_id as i64;

        let report = sqlx::query!(
            "
 SELECT * FROM discord_mod_view WHERE report_id = ?
             ",
            db_id
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(report.map(|r| ModViewModel {
            report_id: r.report_id as u64,
            channel_id: ChannelId(r.channel_id as u64),
            message_id: MessageId(r.message_id as u64),
            preview_archive_id: r.preview_archive_id as u64,
            handler: r.handler.map(|x| UserId(x as u64)),
        }))
    }
}
