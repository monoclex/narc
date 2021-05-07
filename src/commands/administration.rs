use std::time::Duration;

use serenity::{
    client::Context,
    collector::ReactionAction,
    framework::standard::{macros::*, CommandResult},
    model::{
        channel::{Channel, Message, ReactionType},
        id::UserId,
    },
};

use thiserror::Error;

use crate::{database::Database, parsing, state::State};
use serenity::futures::StreamExt;
use serenity::prelude::Mentionable;

#[derive(Debug, Error)]
pub enum SetupCommandError {
    #[error("Message was not sent from within a guild")]
    NoGuild,
    #[error("Timed out")]
    Timeout,
    #[error("An SQL error occurred")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occurred")]
    DiscordError(#[from] serenity::Error),
    #[error("No reports channel was specified")]
    NoReportsChannelSpecified,
    #[error("An invalid reports channel was specified")]
    InvalidReportsChannelSpecified(serenity::Error),
    #[error("Too many channels specified (only one allowed)")]
    TooManyReportsChannelSpecified,
    #[error("Invalid confirmation")]
    InvalidConfirmation(serenity_utils::Error),
    #[error("Configuration rejected")]
    RejectedConfiguration,
}

impl From<serenity_utils::Error> for SetupCommandError {
    fn from(e: serenity_utils::Error) -> Self {
        match e {
            serenity_utils::Error::TimeoutError => Self::Timeout,
            e => Self::InvalidConfirmation(e),
        }
    }
}

#[command]
#[description("Sets up the server for `Narc` to use")]
pub async fn setup(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.ok_or(SetupCommandError::NoGuild)?;

    // put the user in setup mode so they don't react and cause a report
    // this uses `Drop` functionality to set the user out of `in_setup` when
    // the function ends
    let in_setup = InSetup::new(&ctx, &msg.author.id).await;

    let reports_channel = configure_reports_channel(msg, ctx).await?;
    let report_emote = configure_report_emote(msg, ctx).await?;
    let prefix = configure_prefix(msg, ctx).await?;

    let confirmation = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Narc Configuration Confirmation")
                    .field("Reports Channel", reports_channel.mention(), true)
                    .field("Report Emoji", format!("{}", report_emote), true)
                    .field("Narc Prefix", &prefix, true)
            })
        })
        .await?;

    let confirmed =
        serenity_utils::prompt::yes_or_no_prompt(ctx, &confirmation, &msg.author, 30.0).await?;

    if !confirmed {
        return Err(SetupCommandError::RejectedConfiguration)?;
    }

    let read = ctx.data.read().await;
    let db = read.get::<Database>().unwrap();

    db.configure_server(guild_id, report_emote, reports_channel.id(), Some(&prefix))
        .await?;

    std::mem::drop(in_setup);

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Completed!")
                    .description("Narc has been successfully configured.")
            })
        })
        .await?;

    Ok(())
}

async fn configure_reports_channel(
    msg: &Message,
    ctx: &Context,
) -> Result<Channel, SetupCommandError> {
    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configure Narc (1/3)").field(
                    "Reports Channel",
                    "Please type the channel that reports will be sent to",
                    false,
                )
            })
        })
        .await?;

    let reports_channel_response =
        serenity_utils::prompt::message_prompt_content(ctx, msg, &msg.author, 30.0)
            .await
            .ok_or(SetupCommandError::Timeout)?;

    log::info!("user said {} for chanel", reports_channel_response.as_str());
    let reports_channel_mentions = parsing::channel_mention(reports_channel_response.as_str());

    let reports_channel_id = match reports_channel_mentions.len() {
        0 => Err(SetupCommandError::NoReportsChannelSpecified),
        1 => Ok(reports_channel_mentions.into_iter().next().unwrap()),
        _ => Err(SetupCommandError::TooManyReportsChannelSpecified),
    }?;

    let reports_channel = reports_channel_id
        .to_channel(&ctx)
        .await
        .map_err(|e| SetupCommandError::InvalidReportsChannelSpecified(e))?;

    Ok(reports_channel)
}

async fn configure_report_emote(
    msg: &Message,
    ctx: &Context,
) -> Result<ReactionType, SetupCommandError> {
    let emote_prompt = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configure Narc (2/3)").field(
                    "Report Emote",
                    "Please react with the emote to use for reports. Suggested: 🚩",
                    false,
                )
            })
        })
        .await?;

    // TODO: accept response message with emote as well
    let mut collector = msg
        .author
        .await_reactions(&ctx)
        .message_id(emote_prompt.id)
        .timeout(Duration::from_secs(30))
        .await;

    while let Some(emote) = collector.next().await {
        if let ReactionAction::Added(reaction) = emote.as_ref() {
            return Ok(reaction.emoji.to_owned());
        }
    }

    Err(SetupCommandError::Timeout)
}

async fn configure_prefix(msg: &Message, ctx: &Context) -> Result<String, SetupCommandError> {
    let prefix_prompt = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configure Narc (3/3)").field(
                    "Prefix",
                    "What prefix should Narc respond to? Suggested: **`n!`**",
                    false,
                )
            })
        })
        .await?;

    let prompt =
        serenity_utils::prompt::message_prompt_content(ctx, &prefix_prompt, &msg.author, 30.0)
            .await
            .ok_or(SetupCommandError::Timeout)?;

    Ok(prompt)
}

struct InSetup<'ctx> {
    ctx: &'ctx Context,
    user_id: &'ctx UserId,
}

impl<'ctx> InSetup<'ctx> {
    pub async fn new(ctx: &'ctx Context, user_id: &'ctx UserId) -> InSetup<'ctx> {
        let lock = ctx.data.read().await;
        let state = lock.get::<State>().unwrap();
        state
            .mutate_user(&user_id, |user| user.in_setup = true)
            .await;
        Self { ctx, user_id }
    }
}

impl<'ctx> Drop for InSetup<'ctx> {
    fn drop(&mut self) {
        let ctx = self.ctx.clone();
        let user_id = self.user_id.clone();

        tokio::spawn(async move {
            let lock = ctx.data.read().await;
            let state = lock.get::<State>().unwrap();
            state
                .mutate_user(&user_id, |user| user.in_setup = false)
                .await;
        });
    }
}
