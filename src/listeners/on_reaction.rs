use anyhow::Result;
use serenity::model::prelude::*;
use serenity::{client::Context, model::channel::Reaction};

use crate::{
    database::{Database, ServerConfiguration},
    error_handling::handle_err_dms,
    services,
    state::State,
};

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) {
    let data = ctx.data.read().await;
    let state = data.get::<State>().unwrap();

    // `reaction.user_id` is guaranteed to be None if and only if the
    // bot sends a reaction without cache
    // src: https://discord.com/channels/381880193251409931/381912587505500160/840246542715584522
    let user_id = match reaction.user_id {
        Some(u) => u,
        None => return,
    };

    if !state.get_user(&user_id).await.can_make_report() {
        return;
    }

    let db = data.get::<Database>().unwrap();

    if is_report_emoji(&reaction, &db).await {
        let handle_report_result = handle_report(&ctx, &reaction, &db).await;

        match handle_report_result {
            Ok(_) => {}
            Err(error) => {
                log::error!("`handle_report` failed for {:?}: {}", reaction, error);

                let user_id = match reaction.user_id {
                    Some(id) => Ok(id),
                    None => reaction.user(&ctx).await.map(|u| u.id),
                };

                let user_id = match user_id {
                    Ok(u) => u,
                    Err(load_user_error) => {
                        log::error!(
                            "error informing user (on reaction {:?}) about failed report ({}): {}",
                            reaction,
                            error,
                            load_user_error
                        );
                        return;
                    }
                };

                handle_err_dms(
                    &ctx,
                    user_id,
                    None,
                    &error,
                    "Something went wrong when attempting to send in your report.",
                )
                .await;
            }
        }
    }
}

async fn handle_report(ctx: &Context, reaction: &Reaction, db: &Database) -> Result<()> {
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

async fn is_report_emoji(reaction: &Reaction, db: &Database) -> bool {
    let guild_id = match reaction.guild_id {
        Some(guild_id) => guild_id,
        None => {
            // early return: if we do not have a guild ID, we are in in DMs.
            //               when in DMs, we cannot report messages.
            return false;
        }
    };

    let server_config = match db.maybe_load_server_config(guild_id).await {
        Ok(config) => config,
        Err(error) => {
            log::warn!("error loading config for '{:?}': {:?}", guild_id, error);

            // defaults: if we cannot load the server config, we will assume
            //           a default configuration so that users can still
            //           report messages.
            None
        }
    };

    matches_server_emoji(&reaction.emoji, server_config.as_ref())
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
