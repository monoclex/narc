use serenity::model::prelude::*;
use serenity::{client::Context, model::channel::Reaction};
use thiserror::Error;

use crate::{
    database::{
        models::{ReportStatus, ServerConfiguration, ViewModel},
        Database, MakeReportEffect, ReportUpdateError,
    },
    services::{self, MakeReportError},
    state::State,
    view::{self, UpdateViewError},
};

#[derive(Debug, Error)]
pub enum ReactionAddError {
    #[error("An SQL error occurred: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occurred: {0}")]
    DiscordError(#[from] serenity::Error),
    #[error("An error occurred while making the report: {0}")]
    MakeReportError(#[from] MakeReportError),
    #[error("An error occurred while updating the view: {0}")]
    ViewUpdateError(#[from] UpdateViewError),
    #[error("An error occurred while updating the report: {0}")]
    ReportUpdateError(#[from] ReportUpdateError),
    #[error("Timed out")]
    TimedOut,
}

pub async fn reaction_add(ctx: &Context, reaction: &Reaction) -> Result<(), ReactionAddError> {
    // `reaction.user_id` is guaranteed to be None if and only if the
    // bot sends a reaction without cache
    // src: https://discord.com/channels/381880193251409931/381912587505500160/840246542715584522
    let user_id = match reaction.user_id {
        Some(u) => u,
        None => return Ok(()),
    };

    // don't listen to reactions from the bot
    if user_id == ctx.cache.current_user_id().await {
        return Ok(());
    }

    let data = ctx.data.read().await;
    let state = data.get::<State>().unwrap();

    if !state.get_user(&user_id).await.can_make_report() {
        return Ok(());
    }

    let db = data.get::<Database>().unwrap();

    if is_report_emoji(&reaction, &db).await? {
        handle_report(&ctx, &reaction, &db).await?;
    } else if is_refresh_emoji(&reaction.emoji) {
        handle_refresh(&ctx, &reaction, &db).await?;
    } else if is_edit_emoji(&reaction.emoji) {
        handle_edit(&ctx, &reaction, &db).await?;
    } else if is_claim_emoji(&reaction.emoji) {
        handle_claim(&ctx, &reaction, &db).await?;
    } else if is_accept_emoji(&reaction.emoji) {
        handle_finalize(&ctx, &reaction, user_id, &db, true).await?;
    } else if is_reject_emoji(&reaction.emoji) {
        handle_finalize(&ctx, &reaction, user_id, &db, false).await?;
    }

    Ok(())
}

pub async fn reaction_removed(ctx: &Context, reaction: &Reaction) -> Result<(), ReactionAddError> {
    // `reaction.user_id` is guaranteed to be None if and only if the
    // bot sends a reaction without cache
    // src: https://discord.com/channels/381880193251409931/381912587505500160/840246542715584522
    let user_id = match reaction.user_id {
        Some(u) => u,
        None => return Ok(()),
    };

    // don't listen to reactions from the bot
    if user_id == ctx.cache.current_user_id().await {
        return Ok(());
    }

    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();

    if is_claim_emoji(&reaction.emoji) {
        handle_claim(&ctx, &reaction, &db).await?;
    }

    Ok(())
}

async fn handle_report(
    ctx: &Context,
    reaction: &Reaction,
    db: &Database,
) -> Result<(), ReactionAddError> {
    reaction.delete(&ctx).await?;

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

async fn handle_refresh(
    ctx: &Context,
    reaction: &Reaction,
    db: &Database,
) -> Result<(), ReactionAddError> {
    let view = db
        .load_view_by_message(&reaction.message_id, &reaction.channel_id)
        .await?;

    if let Some(ViewModel::Mod(v)) = view {
        if reaction.guild_id.is_some() {
            reaction.delete(&ctx).await?;
        }

        view::update_report_view(&ctx, &db, MakeReportEffect::Updated(v.report_id)).await?
    }

    Ok(())
}

async fn handle_edit(
    ctx: &Context,
    reaction: &Reaction,
    db: &Database,
) -> Result<(), ReactionAddError> {
    let view = db
        .load_view_by_message(&reaction.message_id, &reaction.channel_id)
        .await?;

    let view = match view {
        Some(ViewModel::User(model)) => model,
        // if we react with :pencil: to a mod view model, it shouldn't do anything
        Some(_) => return Ok(()),
        // reacting with :pencil: to regular messages does nothing
        None => return Ok(()),
    };

    // can't delete reactions in DMs, in DMs if guild is none
    if reaction.guild_id.is_some() {
        reaction.delete(&ctx).await?;
    }

    let user = reaction.user(&ctx).await?;

    let msg = reaction
        .channel_id
        .send_message(&ctx, |m| m.content("Type the reason for your report:"))
        .await?;

    let reasoning = crate::serenity_utils::prompt::message_prompt_content(&ctx, &msg, &user, 30.0)
        .await
        .ok_or(ReactionAddError::TimedOut)?;

    services::update_report_reason(&ctx, &db, view.report_id, reasoning).await?;

    reaction
        .channel_id
        .send_message(&ctx, |m| m.content("✅ Your report has been updated"))
        .await?;

    Ok(())
}

async fn handle_claim(
    ctx: &Context,
    reaction: &Reaction,
    db: &Database,
) -> Result<(), ReactionAddError> {
    let view = db
        .load_view_by_message(&reaction.message_id, &reaction.channel_id)
        .await?;

    let view = match view {
        Some(ViewModel::Mod(model)) => model,
        // if we react to a user view model, it shouldn't do anything
        Some(_) => return Ok(()),
        // reacting to regular messages does nothing
        None => return Ok(()),
    };

    let report = match db.load_report(view.report_id).await? {
        Some(r) => r,
        None => {
            return Err(ReactionAddError::ViewUpdateError(
                UpdateViewError::ReportDoesntExist,
            ))
        }
    };

    match report.status {
        ReportStatus::Unhandled => {}
        ReportStatus::Reviewing => {}
        // if the report is already accepted/rejected don't put it back into reviewing
        _ => return Ok(()),
    };

    let claim_reactions = match (reaction.message(&ctx).await?.reactions.iter())
        .find(|e| is_claim_emoji(&e.reaction_type))
    {
        Some(r) => r.count,
        // a message without the reaction
        None => return Ok(()),
    };

    // one reaction is the bot, the other are the people
    let new_status = match claim_reactions {
        x if x >= 2 => ReportStatus::Reviewing,
        x if x < 2 => ReportStatus::Unhandled,
        x => panic!("x = {}: x >= 2 == false, x < 2 == false, wtf?", x),
    };

    services::update_report_status(&ctx, &db, report.id, new_status).await?;

    Ok(())
}

async fn handle_finalize(
    ctx: &Context,
    reaction: &Reaction,
    reaction_user: UserId,
    db: &Database,
    is_accepted: bool,
) -> Result<(), ReactionAddError> {
    let view = db
        .load_view_by_message(&reaction.message_id, &reaction.channel_id)
        .await?;

    let view = match view {
        Some(ViewModel::Mod(model)) => model,
        // if we react to a user view model, it shouldn't do anything
        Some(_) => return Ok(()),
        // reacting to regular messages does nothing
        None => return Ok(()),
    };

    let report = match db.load_report(view.report_id).await? {
        Some(r) => r,
        None => {
            return Err(ReactionAddError::ViewUpdateError(
                UpdateViewError::ReportDoesntExist,
            ))
        }
    };

    reaction.delete(&ctx).await?;

    let new_status = match is_accepted {
        true => ReportStatus::Accepted,
        false => ReportStatus::Denied,
    };

    db.update_mod_view_handler(report.id, reaction_user).await?;
    services::update_report_status(&ctx, &db, report.id, new_status).await?;

    if report.status != new_status {
        if let Some(user_model) = db.load_user_view(report.id).await? {
            let dms = report.accuser_user_id.create_dm_channel(&ctx).await?;

            dms.send_message(&ctx, |m| {
                m.content(format!(
                    "Your report (#{}) has been {}!",
                    report.id,
                    new_status.into_human_status()
                ))
                .reference_message((dms.id, user_model.message_id))
            })
            .await?;
        }
    }

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

    let server_config = db.get_server_config(&guild_id).await?;

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

fn is_refresh_emoji(emoji: &ReactionType) -> bool {
    is_unicode_emoji(&emoji, "🔄")
}

fn is_edit_emoji(emoji: &ReactionType) -> bool {
    is_unicode_emoji(&emoji, "📝")
}

fn is_claim_emoji(emoji: &ReactionType) -> bool {
    is_unicode_emoji(&emoji, "🛄")
}

fn is_reject_emoji(emoji: &ReactionType) -> bool {
    is_unicode_emoji(&emoji, "❌")
}

fn is_accept_emoji(emoji: &ReactionType) -> bool {
    is_unicode_emoji(&emoji, "✅")
}
