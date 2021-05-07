#![forbid(unsafe_code)]

mod commands;
mod database;
mod error_handling;
mod listeners;
mod parsing;
mod services;
mod view;

use anyhow::Result;
use commands::*;
use database::Database;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use std::collections::HashSet;
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    human_panic::setup_panic!();
    let _ = dotenv::dotenv();
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("expected DATABASE_URL env var");
    let token = std::env::var("DISCORD_TOKEN").expect("expected DISCORD_TOKEN env var");

    let database = Database::connect(database_url.as_str()).await?;

    let http = Http::new_with_token(&token);
    let app_info = http.get_current_application_info().await?;
    let bot_id = app_info.id;

    let administrators = vec![app_info.owner.id].into_iter().collect::<HashSet<_>>();

    let framework = StandardFramework::new()
        .configure(|c| {
            c.on_mention(Some(bot_id))
                .dynamic_prefix(|c, m| Box::pin(commands::dynamic_prefix(c, m)))
                .prefix("")
                .ignore_webhooks(true)
                .ignore_bots(true)
                .no_dm_prefix(true)
                .owners(administrators)
                .case_insensitivity(true)
        })
        .group(&ASSISTANCE_GROUP);

    let mut client = Client::builder(token)
        .event_handler(listeners::Listener)
        .framework(framework)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<Database>(database);
    }

    client.start().await?;

    Ok(())
}
