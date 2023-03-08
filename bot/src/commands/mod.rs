use crate::{database::Database, error_handling::*};
use serenity::{client::Context, framework::standard::DispatchError, model::channel::*};
use serenity::{
    framework::standard::{macros::*, CommandResult},
    utils::content_safe,
};

mod assistance;
use assistance::*;

mod administration;
use administration::*;

mod help;
pub use help::*;

#[group("Assistance")]
#[description = "Commands that serve to aid users in getting assistance"]
#[commands(report)]
pub struct Assistance;

#[group("Administration")]
#[description = "Commands that are for administrator use only"]
#[commands(setup, protect, protected)]
pub struct Administration;

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
        &[],
    );

    handle_err(&ctx, msg.channel_id, Some(msg.id), &error, error_msg).await;
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, err: DispatchError, command_name: &str) {
    let message: String = match err {
        DispatchError::OnlyForOwners => "This command is only available for owners!".into(),
        DispatchError::LackingPermissions(p) => {
            format!("You are missing the following permissions: {}", p)
        }
        unknown => format!("Unknown dispatch error occurred: {:?}", unknown),
    };

    handle_err(
        &ctx,
        msg.channel_id,
        Some(msg.id),
        &message,
        "A dispatch error occurred",
    )
    .await;
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

    database.get_server_prefix(&guild_id).await.ok()?
}
