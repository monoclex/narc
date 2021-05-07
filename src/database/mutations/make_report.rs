use serenity::model::{channel::Message, prelude::*};
use sqlx::{pool::PoolConnection, Sqlite};

use crate::database::{models::*, Database};

type ReportId = u64;

/// Describes the effects that happened when making the report. This is used
/// to accurately update the view.
pub enum MakeReportEffect {
    Created(ReportId),
    Updated(ReportId),
    Duplicate(ReportId),
}

// TODO: put parameters into a struct and impl methods on that so we're not
// passing os many parameters around
impl Database {
    pub async fn make_report(
        &self,
        guild_id: GuildId,
        user_reporting: &User,
        reported_user: &User,
        reported_message: Option<&Message>,
        report_reason: Option<&str>,
    ) -> Result<MakeReportEffect, sqlx::Error> {
        let mut connection = self.connection.acquire().await?;

        sqlx::query("BEGIN EXCLUSIVE;")
            .execute(&mut connection)
            .await?;

        if let Some(message) = reported_message {
            self.archive_message_in_transaction(&mut connection, message)
                .await?
                .message_id();
        }

        let effect = self
            .create_report(
                &mut connection,
                guild_id,
                &user_reporting,
                &reported_user,
                reported_message,
                report_reason,
            )
            .await?;

        sqlx::query("COMMIT;").execute(&mut connection).await?;

        Ok(effect)
    }

    async fn create_report(
        &self,
        connection: &mut PoolConnection<Sqlite>,
        guild_id: GuildId,
        user_reporting: &User,
        reported_user: &User,
        reported_message: Option<&Message>,
        report_reason: Option<&str>,
    ) -> Result<MakeReportEffect, sqlx::Error> {
        let db_gid = guild_id.0 as i64;
        let db_mid = reported_message.map(|m| m.id.0 as i64);
        let db_aid = user_reporting.id.0 as i64;
        let db_rid = reported_user.id.0 as i64;
        let db_s = Into::<i64>::into(ReportStatus::Unhandled);
        let db_cid = reported_message.map(|m| m.channel_id.0 as i64);

        // ensure no duplicate reports open
        if let Some(message) = reported_message {
            let db_mid = message.id.0 as i64;

            let existing_report =
                Database::fetch_existing_report(db_mid, db_aid, &mut *connection).await?;

            if let Some(existing_report) = existing_report {
                return Ok(MakeReportEffect::Duplicate(existing_report));
            }
        }

        // create the report
        let report = sqlx::query!(
            "
INSERT INTO reports (accuser_user_id, reported_user_id, guild_id, status, channel_id, message_id, reason)
VALUES (?, ?, ?, ?, ?, ?, ?);
            ",
            db_aid,
            db_rid,
            db_gid,
            db_s,
            db_cid,
            db_mid,
            report_reason
        )
        .execute(&mut *connection)
        .await?;

        Ok(MakeReportEffect::Created(report.last_insert_rowid() as u64))
    }

    /// Fetches an existingh report with the same accuser id and message id, to
    /// prevent duplicate reports.
    async fn fetch_existing_report(
        message_id: i64,
        accuser_user_id: i64,
        connection: &mut PoolConnection<Sqlite>,
    ) -> Result<Option<ReportId>, sqlx::Error> {
        let existing_report = sqlx::query!(
            "
SELECT * FROM reports
WHERE message_id = ?
  AND accuser_user_id = ?
                ",
            message_id,
            accuser_user_id
        )
        .fetch_optional(&mut *connection)
        .await?;

        Ok(existing_report.map(|r| r.id as u64))
    }
}
