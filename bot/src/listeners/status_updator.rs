use serenity::{client::Context, model::prelude::*};

pub async fn ready(ctx: &Context, data_about_bot: &Ready) {
    let guilds = data_about_bot.guilds.len();
    StatusUpdator(ctx).update_presence_with_guilds(guilds).await;
}

pub async fn guild_create(ctx: &Context) {
    StatusUpdator(ctx).update_presence().await;
}

pub async fn guild_delete(ctx: &Context) {
    StatusUpdator(ctx).update_presence().await;
}

/// Handles updating the status of the bot to reflect how many guilds it is in.
pub struct StatusUpdator<'ctx>(&'ctx Context);

impl StatusUpdator<'_> {
    pub async fn update_presence(&self) {
        let guilds = self.0.cache.guilds().len();
        self.update_presence_with_guilds(guilds).await;
    }

    pub async fn update_presence_with_guilds(&self, guilds: usize) {
        let status = format!("n!help | 👁️ {} guilds", guilds);
        let activity = Activity::playing(status);

        self.0
            .set_presence(Some(activity), OnlineStatus::Online)
            .await;
    }
}
