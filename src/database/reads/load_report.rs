use serenity::model::id::*;

use crate::database::{models::*, Database};

impl Database {
    pub async fn load_report(&self, report_id: u64) -> Result<Option<ReportModel>, sqlx::Error> {
        let db_id = report_id as i64;

        let report = sqlx::query!(
            "
SELECT * FROM reports WHERE id = ?
            ",
            db_id
        )
        .fetch_optional(&self.connection)
        .await?;

        Ok(report.map(|r| ReportModel {
            id: r.id as u64,
            accuser_user_id: UserId(r.accuser_user_id as u64),
            reported_user_id: UserId(r.reported_user_id as u64),
            guild_id: GuildId(r.guild_id as u64),
            status: r.status.into(),
            message_id: r.message_id.map(|x| MessageId(x as u64)),
            channel_id: r.channel_id.map(|x| ChannelId(x as u64)),
            reason: r.reason,
        }))
    }
}
