use crate::{database::Database, view};
use serenity::{client::Context, model::id::*};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MakeReportError {
    #[error("Cannot submit report for unconfigured server")]
    UnconfiguredServer,
    #[error("An SQL error occured")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occured")]
    DiscordError(#[from] serenity::Error),
}

pub async fn make_report(
    ctx: &Context,
    db: &Database,
    guild_id: GuildId,
    accuser_user_id: UserId,
    reported_user_id: UserId,
    reported_channel_id: Option<ChannelId>,
    reported_message_id: Option<MessageId>,
    report_reason: Option<&str>,
) -> Result<(), MakeReportError> {
    // before we make a report, lets ensure that the server is configured
    if db.maybe_load_server_config(guild_id).await?.is_none() {
        return Err(MakeReportError::UnconfiguredServer);
    }

    let user_reporting = accuser_user_id.to_user(&ctx).await?;
    let reported_user = reported_user_id.to_user(&ctx).await?;

    let mut reported_message = None;

    if let Some(c) = reported_channel_id {
        if let Some(m) = reported_message_id {
            reported_message = Some(c.message(&ctx, m).await?);
        }
    }

    let effect = db
        .make_report(
            guild_id,
            &user_reporting,
            &reported_user,
            reported_message.as_ref(),
            report_reason,
        )
        .await?;
    view::update_report_view(ctx, &db, effect).await;

    Ok(())
}
