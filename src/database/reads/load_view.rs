use serenity::model::id::*;

use crate::database::{models::*, Database};

impl Database {
    pub async fn load_view_by_message(
        &self,
        message_id: &MessageId,
        channel_id: &ChannelId,
    ) -> Result<Option<ViewModel>, sqlx::Error> {
        let db_mid = message_id.0 as i64;
        let db_cid = channel_id.0 as i64;

        let record = sqlx::query!(
            "
SELECT * FROM discord_mod_view
WHERE message_id = ?
  AND channel_id = ?;
            ",
            db_mid,
            db_cid,
        )
        .fetch_optional(&self.connection)
        .await?;

        if let Some(r) = record {
            // TODO: deduplicate mapping code
            return Ok(Some(ViewModel::Mod(ModViewModel {
                report_id: r.report_id as u64,
                channel_id: ChannelId(r.channel_id as u64),
                message_id: MessageId(r.message_id as u64),
                preview_archive_id: r.preview_archive_id as u64,
                handler: r.handler.map(|x| UserId(x as u64)),
            })));
        }

        let record = sqlx::query!(
            "
SELECT * FROM discord_user_view
WHERE message_id = ?;
            ",
            db_mid,
        )
        .fetch_optional(&self.connection)
        .await?;

        if let Some(r) = record {
            // TODO: deduplicate mapping code
            return Ok(Some(ViewModel::User(UserViewModel {
                report_id: r.report_id as u64,
                message_id: MessageId(r.message_id as u64),
                status: r.status.into(),
            })));
        }

        Ok(None)
    }
}
