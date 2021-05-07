use serenity::client::{Context, EventHandler};
use serenity::{async_trait, model::prelude::*};

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
        on_reaction::reaction_add(&ctx, &reaction).await;
        //     match self.make_handler(ctx).handle_reaction(reaction).await {
        //         Ok(_) => {}
        //         Err(crate::bot::HandleReactionError::LoadError(sqlx::Error::RowNotFound)) => {
        //             log::warn!("`handle_reaction` no server config")
        //         }
        //         Err(error) => log::error!("unable to handle `handle_reaction`: {:#?}", error),
        //     };
    }
}
