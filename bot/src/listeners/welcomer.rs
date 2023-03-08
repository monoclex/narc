use std::collections::HashSet;

use serenity::{
    client::Context,
    model::{
        guild::{Guild, UnavailableGuild},
        id::GuildId,
    },
};

use crate::{database::Database, error_handling::handle_err_dms};
use thiserror::Error;

pub async fn ready(ctx: &Context, guilds: impl Iterator<Item = GuildId>) {
    let guilds = guilds.map(|gid| gid.0).collect::<HashSet<_>>();

    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();

    let servers = match db.get_welcomed_servers().await {
        Ok(servers) => servers,
        Err(error) => {
            log::error!("error while pruning welcomes: {}", error);
            panic!("error while pruning welcomes: {}", error);
        }
    };

    let servers_to_send_welcome_messages_to = guilds.difference(&servers);
    for server in servers_to_send_welcome_messages_to {
        if let Err(error) = welcome(&ctx, &db, &GuildId(*server)).await {
            log::error!("error while welcoming server: {}", error);
        }
    }
}

pub async fn guild_create(ctx: &Context, guild: &Guild) {
    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();

    // even if we've welcomed them in the past, welcome them again
    if let Err(error) = welcome(&ctx, &db, &guild.id).await {
        handle_err_dms(
            ctx,
            guild.owner_id,
            None,
            &error,
            "An error occurred while welcoming you",
        )
        .await;
    }
}

pub async fn guild_delete(ctx: &Context, incomplete: &UnavailableGuild) {
    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();

    match db.delete_welcome(&incomplete.id).await {
        Ok(_) => {}
        Err(error) => {
            log::error!("error while deleting welcome: {}", error)
        }
    };
}

#[derive(Debug, Error)]
enum WelcomeError {
    #[error("An SQL error occurred: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occurred: {0}")]
    DiscordError(#[from] serenity::Error),
}

async fn welcome(ctx: &Context, db: &Database, server: &GuildId) -> Result<(), WelcomeError> {
    let owner_dms = server
        .to_partial_guild(&ctx)
        .await?
        .owner_id
        .create_dm_channel(&ctx)
        .await?;

    owner_dms
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Welcome to Narc!").field(
                    "Getting Started",
                    "
Make sure you leave your DMs to bots enabled so you can receive error information.
Just run `n!setup`, and follow the instructions to get started.
"
                    .trim(),
                    false,
                )
            })
        })
        .await?;

    db.make_welcome(server).await?;

    Ok(())
}
