use serenity::client::{Context, EventHandler};
use serenity::{async_trait, model::prelude::*};

use crate::error_handling::handle_err;

mod on_reaction;
mod status_updator;

pub struct Listener;

#[async_trait]
impl EventHandler for Listener {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        status_updator::ready(&ctx, &data_about_bot).await;
    }

    async fn guild_create(&self, ctx: Context, _guild: Guild, _is_new: bool) {
        status_updator::guild_create(&ctx).await;
    }

    async fn guild_delete(
        &self,
        ctx: Context,
        _incomplete: GuildUnavailable,
        _full: Option<Guild>,
    ) {
        status_updator::guild_delete(&ctx).await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        match on_reaction::reaction_add(&ctx, &reaction).await {
            Err(error) => {
                handle_err(
                    &ctx,
                    reaction.channel_id,
                    None,
                    &error,
                    "An error occurred during your reaction",
                )
                .await
            }
            _ => {}
        }
    }
}
