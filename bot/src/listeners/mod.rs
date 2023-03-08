use serenity::client::{Context, EventHandler};
use serenity::{async_trait, model::prelude::*};

use crate::error_handling::handle_err;

mod on_msg;
mod on_reaction;
mod status_updator;
mod welcomer;

pub struct Listener;

#[async_trait]
impl EventHandler for Listener {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        status_updator::ready(&ctx, &data_about_bot).await;
        welcomer::ready(&ctx, data_about_bot.guilds.iter().map(|s| s.id)).await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            status_updator::guild_create(&ctx).await;
            welcomer::guild_create(&ctx, &guild).await;
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, _full: Option<Guild>) {
        status_updator::guild_delete(&ctx).await;
        welcomer::guild_delete(&ctx, &incomplete).await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if let Err(error) = on_reaction::reaction_add(&ctx, &reaction).await {
            handle_err(
                &ctx,
                reaction.channel_id,
                None,
                &error,
                "An error occurred during your reaction",
            )
            .await
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        if let Err(error) = on_reaction::reaction_removed(&ctx, &removed_reaction).await {
            handle_err(
                &ctx,
                removed_reaction.channel_id,
                None,
                &error,
                "An error occurred during your reaction",
            )
            .await
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        println!("msg is {message:?}");
        if let Err(error) = on_msg::message(&ctx, &message).await {
            handle_err(
                &ctx,
                message.channel_id,
                None,
                &error,
                "An error occurred during your reaction",
            )
            .await
        }
    }
}
