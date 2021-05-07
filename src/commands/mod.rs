use crate::{database::Database, error_handling::*};
use serenity::{client::Context, model::channel::*};
use serenity::{
    framework::standard::{macros::*, CommandResult},
    utils::content_safe,
};

use assistance::*;
mod assistance;

#[group("Assistance")]
#[description = "Commands that serve to aid users in getting assistance"]
#[commands(report)]
pub struct Assistance;

#[hook]
pub async fn after(ctx: &Context, msg: &Message, cmd: &str, err: CommandResult) {
    let error = match err {
        Ok(_) => {
            let reaction = msg
                .react(&ctx, ReactionType::Unicode("✅".to_owned()))
                .await;

            if let Err(err) = reaction {
                log::error!("error reaction with :white_check_mark: to user: {:?}", err);
            }
            return;
        }
        Err(err) => err,
    };

    let error_msg = content_safe(
        &ctx,
        format!(
            "Something went wrong when attempting to execute the command '{}'",
            cmd
        ),
        &Default::default(),
    )
    .await;

    handle_err(&ctx, msg.channel_id, Some(msg.id), &error, error_msg).await;
}

pub async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    // we always want the `n!` prefix to be overridable, so we use
    // `dynamic_prefix` rather than shelling out to `.prefix()` in the
    // framework
    Some(
        actual_dynamic_prefix(ctx, msg)
            .await
            .unwrap_or("n!".to_owned()),
    )
}

async fn actual_dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    let guild_id = msg.guild_id?;

    let read = ctx.data.read().await;
    let database = read.get::<Database>().unwrap();

    database.load_server_prefix(guild_id).await.ok()?
}
