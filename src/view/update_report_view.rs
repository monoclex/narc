use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{channel::ReactionType, id::*, prelude::User},
    prelude::Mentionable,
};

use crate::database::{models::*, Database, MakeReportEffect};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateViewError {
    #[error("Report does not exist in database.")]
    ReportDoesntExist,
    #[error("An SQL error occurred: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occurred: {0}")]
    DiscordError(#[from] serenity::Error),
    #[error("This server has not been configured yet")]
    UnconfiguredServer,
}

// TODO: simplify verbose error handling by propagating it up
pub async fn update_report_view(
    ctx: &Context,
    db: &Database,
    effect: MakeReportEffect,
) -> Result<(), UpdateViewError> {
    let report_id = match effect {
        MakeReportEffect::Created(id) => id,
        MakeReportEffect::Updated(id) => id,
        MakeReportEffect::Duplicate(_) => return Ok(()),
    };

    let report = db
        .load_report(report_id)
        .await?
        .ok_or(UpdateViewError::ReportDoesntExist)?;

    update_mod_view(&ctx, &db, &report).await?;
    update_user_view(&ctx, &db, &report).await?;

    Ok(())
}

async fn update_user_view(
    ctx: &Context,
    db: &Database,
    report: &ReportModel,
) -> Result<(), UpdateViewError> {
    let view = db.load_user_view(report.id).await?;
    let dms = report.accuser_user_id.create_dm_channel(&ctx).await?;

    let channel_name = match report.channel_id {
        Some(c) => c.name(&ctx).await,
        None => None,
    };

    let msg = match view {
        Some(view) => {
            dms.edit_message(&ctx, view.message_id, |m| {
                m.embed(|e| display_user_view(&report, e, channel_name))
            })
            .await?
        }
        None => {
            dms.send_message(&ctx, |m| {
                m.embed(|e| display_user_view(&report, e, channel_name))
            })
            .await?
        }
    };

    msg.react(&ctx, ReactionType::Unicode("🔄".to_owned()))
        .await?;

    msg.react(&ctx, ReactionType::Unicode("📝".to_owned()))
        .await?;

    let updated_model = UserViewModel {
        report_id: report.id,
        message_id: msg.id,
        status: report.status,
    };

    db.save_user_view(updated_model).await?;

    Ok(())
}

fn display_user_view<'a, 'b>(
    report: &ReportModel,
    e: &'a mut CreateEmbed,
    channel_name: Option<String>,
) -> &'a mut CreateEmbed {
    e.title(format!("Report (ID #{})", report.id));

    e.field("Reported User", report.reported_user_id.mention(), true)
        .field("Status", format!("{:?}", report.status), true);

    if let Some(url) = report.url() {
        e.field(
            "Location",
            format!(
                "[#{}]({})",
                channel_name.unwrap_or("error fetching channel name".to_owned()),
                url
            ),
            true,
        );
    }

    e.field(
        "Provided Reason",
        report
            .reason
            .as_deref()
            .unwrap_or("No reason provided! React with 📝 to provide one."),
        false,
    )
}

async fn update_mod_view(
    ctx: &Context,
    db: &Database,
    report: &ReportModel,
) -> Result<(), UpdateViewError> {
    let view = db.load_mod_view(report.id).await?;
    let maybe_config = db.maybe_load_server_config(report.guild_id).await?;
    let config = maybe_config.ok_or(UpdateViewError::UnconfiguredServer)?;

    // TODO: handle a changed reports channel and whatnot?
    let channel_id = ChannelId(config.reports_channel);
    let channel_name = match report.channel_id {
        Some(c) => c.name(&ctx).await,
        None => None,
    };

    let reporter = report.accuser_user_id.to_user(&ctx).await?;
    let reported = report.reported_user_id.to_user(&ctx).await?;

    // TODO: handle if the report message got deleted
    let msg = match view {
        Some(view) => {
            channel_id
                .edit_message(&ctx, view.message_id, |m| {
                    m.embed(|e| display_mod_view(&report, e, channel_name, reported, reporter))
                })
                .await?
        }
        None => {
            channel_id
                .send_message(&ctx, |m| {
                    m.embed(|e| display_mod_view(&report, e, channel_name, reported, reporter))
                })
                .await?
        }
    };

    msg.react(&ctx, ReactionType::Unicode("🔄".to_owned()))
        .await?;

    msg.react(&ctx, ReactionType::Unicode("🛄".to_owned()))
        .await?;

    msg.react(&ctx, ReactionType::Unicode("❌".to_owned()))
        .await?;

    msg.react(&ctx, ReactionType::Unicode("✅".to_owned()))
        .await?;

    let updated_model = ModViewModel {
        report_id: report.id,
        channel_id,
        message_id: msg.id,
        // TODO: handle valid message id in archive
        preview_archive_id: 0,
        // TODO: handle mods clicking buttons n stuff
        handler: None,
    };

    db.save_mod_view(updated_model).await?;

    Ok(())
}

fn display_mod_view<'a, 'b>(
    report: &ReportModel,
    e: &'a mut CreateEmbed,
    channel_name: Option<String>,
    reported: User,
    reporter: User,
) -> &'a mut CreateEmbed {
    let reporter_user = reporter;
    let reported_user = reported;

    let avatar_url = reporter_user
        .avatar_url()
        .unwrap_or_else(|| reporter_user.default_avatar_url());
    let reported_user_mention = reported_user.mention();
    let reporter_user_mention = reporter_user.mention();

    let preview = Option::<String>::None;

    e.author(|a| {
        a.icon_url(avatar_url)
            .name(format!("Report (ID #{})", report.id))
    })
    .field("Accused User", reported_user_mention, true)
    .field("Reported By", reporter_user_mention, true)
    .field("Status", "Unclaimed", true);

    if let Some(preview) = preview {
        e.field("Preview", preview, false);
    }

    if let Some(location_link) = report.url() {
        e.field(
            "Location",
            format!(
                "[#{}]({})",
                channel_name.unwrap_or("error fetching channel name".to_owned()),
                location_link
            ),
            true,
        );
    }

    e
}
