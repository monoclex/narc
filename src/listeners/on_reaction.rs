use serenity::model::prelude::*;
use serenity::{client::Context, model::channel::Reaction};
use thiserror::Error;

use crate::{
    database::{Database, ServerConfiguration},
    services::{self, MakeReportError},
    state::State,
};

#[derive(Debug, Error)]
pub enum ReactionAddError {
    #[error("An SQL error occurred: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occurred: {0}")]
    DiscordError(#[from] serenity::Error),
    #[error("An error occurred while making the report: {0}")]
    MakeReportError(#[from] MakeReportError),
}

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) -> Result<(), ReactionAddError> {
    let data = ctx.data.read().await;
    let state = data.get::<State>().unwrap();

    // `reaction.user_id` is guaranteed to be None if and only if the
    // bot sends a reaction without cache
    // src: https://discord.com/channels/381880193251409931/381912587505500160/840246542715584522
    let user_id = match reaction.user_id {
        Some(u) => u,
        None => return Ok(()),
    };

    if !state.get_user(&user_id).await.can_make_report() {
        return Ok(());
    }

    let db = data.get::<Database>().unwrap();

    if is_report_emoji(&reaction, &db).await? {
        reaction.delete(&ctx).await?;
        handle_report(&ctx, &reaction, &db).await?;
    }

    Ok(())
}

async fn handle_report(
    ctx: &Context,
    reaction: &Reaction,
    db: &Database,
) -> Result<(), ReactionAddError> {
    // TODO: pass along `guild_id` or something?
    let guild_id = reaction.guild_id.unwrap(); // guaranteed by `is_report_emoji`

    let reported_message = reaction.message(ctx).await?;
    let user_reporting = reaction.user(&ctx).await?;

    services::make_report(
        &ctx,
        &db,
        guild_id,
        user_reporting.id,
        reported_message.author.id,
        Some(reaction.channel_id),
        Some(reaction.message_id),
        None,
    )
    .await?;

    Ok(())
}

async fn is_report_emoji(reaction: &Reaction, db: &Database) -> Result<bool, sqlx::Error> {
    let guild_id = match reaction.guild_id {
        Some(guild_id) => guild_id,
        None => {
            // early return: if we do not have a guild ID, we are in in DMs.
            //               when in DMs, we cannot report messages.
            return Ok(false);
        }
    };

    let server_config = db.maybe_load_server_config(guild_id).await?;

    Ok(matches_server_emoji(
        &reaction.emoji,
        server_config.as_ref(),
    ))
}

fn matches_server_emoji(emoji: &ReactionType, server_config: Option<&ServerConfiguration>) -> bool {
    match server_config {
        Some(config) => config.matches_emoji(&emoji),

        // defaults: if there is no specific server configuration, `🚩` is the
        //           default report emoji.
        None => is_unicode_emoji(emoji, "🚩"),
    }
}

fn is_unicode_emoji(reaction_type: &ReactionType, unicode_str: &str) -> bool {
    // TODO: use `matches!(emoji, ReactionType::Unicode("🚩"))` when possible.
    match reaction_type {
        ReactionType::Unicode(ref str) => str == unicode_str,
        _ => false,
    }
}
