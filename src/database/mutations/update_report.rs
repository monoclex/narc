use crate::database::{models::ReportStatus, Database};
use serenity::model::id::UserId;
use thiserror::Error;

type ReportId = u64;

#[derive(Debug, Error)]
pub enum ReportUpdateError {
    #[error("An SQL error occurred: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A surprising amount of rows were updated ({0} expected 1)")]
    SurprisingRowUpdateCount(u64),
}

impl Database {
    pub async fn update_report<S: ToString>(
        &self,
        report_id: ReportId,
        reason: Option<S>,
        status: Option<ReportStatus>,
    ) -> Result<(), ReportUpdateError> {
        let db_id = report_id as i64;
        let db_r = reason.map(|s| s.to_string());
        let db_s = status.map(|s| Into::<i64>::into(s));

        let result = sqlx::query!(
            "
UPDATE reports
SET reason = COALESCE(?, reason),
    status = COALESCE(?, status)
WHERE id = ?;
            ",
            db_r,
            db_s,
            db_id,
        )
        .execute(&self.connection)
        .await?;

        if result.rows_affected() != 1 {
            return Err(ReportUpdateError::SurprisingRowUpdateCount(
                result.rows_affected(),
            ));
        }

        Ok(())
    }

    pub async fn update_mod_view_handler(
        &self,
        report_id: ReportId,
        moderator: UserId,
    ) -> Result<(), ReportUpdateError> {
        let db_id = report_id as i64;
        let db_h = moderator.0 as i64;

        let result = sqlx::query!(
            "
UPDATE discord_mod_view
SET handler = ?
WHERE report_id = ?;
            ",
            db_h,
            db_id,
        )
        .execute(&self.connection)
        .await?;

        if result.rows_affected() != 1 {
            return Err(ReportUpdateError::SurprisingRowUpdateCount(
                result.rows_affected(),
            ));
        }

        Ok(())
    }
}
