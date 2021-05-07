use serenity::model::channel::Message;
use sqlx::{pool::PoolConnection, Sqlite};

use crate::database::Database;

type ArchiveMessageId = u64;

/// Describes the effects that happened when making the report. This is used
/// to accurately update the view.
pub enum ArchiveMessageEffect {
    Archived(ArchiveMessageId),
    None(ArchiveMessageId),
}

impl ArchiveMessageEffect {
    pub fn message_id(&self) -> ArchiveMessageId {
        match self {
            ArchiveMessageEffect::Archived(id) => *id,
            ArchiveMessageEffect::None(id) => *id,
        }
    }
}

impl Database {
    // pub async fn archive_message(
    //     &self,
    //     message: &Message,
    // ) -> Result<ArchiveMessageEffect, sqlx::Error> {
    //     let mut connection = self.connection.acquire().await?;

    //     // since we are creating reports, we want exclusive access to the DB
    //     // while we update it
    //     sqlx::query("BEGIN EXCLUSIVE;")
    //         .execute(&mut connection)
    //         .await?;

    //     let effect = self
    //         .archive_message_in_transaction(&mut connection, message)
    //         .await?;

    //     sqlx::query("COMMIT;").execute(&mut connection).await?;

    //     Ok(effect)
    // }

    pub(super) async fn archive_message_in_transaction(
        &self,
        connection: &mut PoolConnection<Sqlite>,
        message: &Message,
    ) -> Result<ArchiveMessageEffect, sqlx::Error> {
        let db_msg_id = message.id.0 as i64;

        // get the most recent copy of the archived message
        let archived_message = sqlx::query!(
            "
SELECT * FROM message_archive
WHERE message_id = ?
ORDER BY id DESC
LIMIT 1;
            ",
            db_msg_id
        )
        .fetch_optional(&mut *connection)
        .await?;

        match archived_message {
            Some(archived_message) => {
                let need_update = archived_message.content != message.content;

                if !need_update {
                    return Ok(ArchiveMessageEffect::None(archived_message.id as u64));
                }

                // we will need to update it at this point - fall into the case
                // where we update it
            }
            None => {
                // don't do anything, fall into the case where we archive the message
            }
        };

        let db_content = message.content.clone();
        let archived_message = sqlx::query!(
            "
INSERT INTO message_archive (message_id, content)
VALUES (?, ?)
            ",
            db_msg_id,
            db_content
        )
        .execute(&mut *connection)
        .await?;

        Ok(ArchiveMessageEffect::Archived(
            archived_message.last_insert_rowid() as u64,
        ))
    }
}
