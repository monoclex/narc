use std::time::Duration;

use serenity::{
    client::Context,
    collector::ReactionAction,
    framework::standard::{macros::*, CommandResult},
    model::{
        channel::{Channel, Message, ReactionType},
        id::UserId,
        prelude::User,
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
    #[error("An SQL error occurred: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occurred: {0}")]
    DiscordError(#[from] serenity::Error),
    #[error("The message you sent was unable to be parsed for an emoji. Try reacting to the message instead.")]
    UnparseableEmoji,
    #[error("No reports channel was specified")]
    NoReportsChannelSpecified,
    #[error("An invalid reports channel was specified")]
    InvalidReportsChannelSpecified(serenity::Error),
    #[error("Too many channels specified (only one allowed)")]
    TooManyReportsChannelSpecified,
    #[error("Invalid confirmation: {0}")]
    InvalidConfirmation(crate::serenity_utils::Error),
    #[error("Configuration rejected")]
    RejectedConfiguration,
}

impl From<crate::serenity_utils::Error> for SetupCommandError {
    fn from(e: crate::serenity_utils::Error) -> Self {
        match e {
            crate::serenity_utils::Error::TimeoutError => Self::Timeout,
            e => Self::InvalidConfirmation(e),
        }
    }
}

#[command]
#[required_permissions(ADMINISTRATOR)]
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
        crate::serenity_utils::prompt::yes_or_no_prompt(ctx, &confirmation, &msg.author, 30.0)
            .await?;

    if !confirmed {
        return Err(SetupCommandError::RejectedConfiguration.into());
    }

    let read = ctx.data.read().await;
    let db = read.get::<Database>().unwrap();

    db.save_server_configuration(guild_id, report_emote, reports_channel.id(), Some(&prefix))
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
        crate::serenity_utils::prompt::message_prompt_content(ctx, msg, &msg.author, 30.0)
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
        .map_err(SetupCommandError::InvalidReportsChannelSpecified)?;

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
                    "Please react or type the emote to use for reports. Suggested: 🚩",
                    false,
                )
            })
        })
        .await?;

    let text_response = accept_response(&emote_prompt, &msg.author, &ctx);
    let reaction_response = accept_reaction(&emote_prompt, &msg.author, &ctx);

    let result = tokio::select! {
        result = text_response => {result},
        result = reaction_response => {result}
    };
    return result;

    async fn accept_response(
        prompt: &Message,
        author: &User,
        ctx: &Context,
    ) -> Result<ReactionType, SetupCommandError> {
        let text_response =
            crate::serenity_utils::prompt::message_prompt_content(ctx, &prompt, &author, 30.0)
                .await
                .ok_or(SetupCommandError::Timeout)?;

        if let Some(twemoji) = (text_response.chars())
            .take(1)
            .find(|&c| unic::emoji::char::is_emoji(c))
        {
            return Ok(ReactionType::Unicode(twemoji.to_string()));
        }

        match serenity::utils::parse_emoji(text_response) {
            Some(emoji) => Ok(dbg!(emoji.into())),
            None => Err(SetupCommandError::UnparseableEmoji),
        }
    }

    async fn accept_reaction(
        prompt: &Message,
        author: &User,
        ctx: &Context,
    ) -> Result<ReactionType, SetupCommandError> {
        let mut collector = author
            .await_reactions(&ctx)
            .message_id(prompt.id)
            .timeout(Duration::from_secs(30))
            .await;

        while let Some(emote) = collector.next().await {
            if let ReactionAction::Added(reaction) = emote.as_ref() {
                return Ok(reaction.emoji.to_owned());
            }
        }

        Err(SetupCommandError::Timeout)
    }
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

    let prompt = crate::serenity_utils::prompt::message_prompt_content(
        ctx,
        &prefix_prompt,
        &msg.author,
        30.0,
    )
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
        let user_id = *self.user_id;

        tokio::spawn(async move {
            let lock = ctx.data.read().await;
            let state = lock.get::<State>().unwrap();
            state
                .mutate_user(&user_id, |user| user.in_setup = false)
                .await;
        });
    }
}
