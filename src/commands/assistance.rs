use serenity::{
    client::Context,
    framework::standard::{macros::*, Args, CommandResult},
    model::channel::Message,
};

use thiserror::Error;

use crate::{database::Database, parsing::FailedUserParse, services::MakeReportError};
use crate::{parsing, services};

#[derive(Debug, Error)]
pub enum ReportCommandError {
    #[error("Message was not sent from within a guild")]
    NoGuild,
    #[error("Error parsing user")]
    UserParseError(#[from] FailedUserParse),
    #[error("Error occured while making report")]
    MakeReportError(#[from] MakeReportError),
}

#[command]
#[aliases("r")]
#[description("Submits a report on a user")]
pub async fn report(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();

    // TODO: check if the user includes a link in their message, and if so, use
    //       the guild the link comes from as the `guild_id` and `reported_message`

    let guild = msg.guild(&ctx).await.ok_or(ReportCommandError::NoGuild)?;

    let name = args.single_quoted::<String>()?;
    let user = parsing::user(&name, &ctx, &guild).await?;

    let reason = args.remains();

    services::make_report(
        &ctx,
        &db,
        guild.id,
        msg.author.id,
        user.user_id(),
        None,
        None,
        reason,
    )
    .await?;

    Ok(())
}
