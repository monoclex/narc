use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{id::*, prelude::User},
    prelude::Mentionable,
};

use crate::database::{models::*, Database, MakeReportEffect};

// TODO: simplify verbose error handling by propagating it up
pub async fn update_report_view(ctx: &Context, db: &Database, effect: MakeReportEffect) {
    let report_id = match effect {
        MakeReportEffect::Created(id) => id,
        MakeReportEffect::Updated(id) => id,
        MakeReportEffect::Duplicate(_) => return,
    };

    let report = match db.load_report(report_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            log::error!("unable to load report {} (wtf?)", report_id);
            return;
        }
        Err(error) => {
            log::error!("unable to load report {}: {}", report_id, error);
            return;
        }
    };

    tokio::join!(
        update_user_view(&ctx, &db, &report),
        update_mod_view(&ctx, &db, &report),
    );
}

async fn update_user_view(ctx: &Context, db: &Database, report: &ReportModel) {
    let view = match db.load_user_view(report.id).await {
        Ok(v) => v,
        Err(error) => {
            log::error!("unable to load user view model {}: {}", report.id, error);
            return;
        }
    };

    let dms = match report.accuser_user_id.create_dm_channel(&ctx).await {
        Ok(x) => x,
        Err(error) => {
            log::error!(
                "unable to start dms for {}: {}",
                report.accuser_user_id,
                error
            );
            return;
        }
    };

    let msg = match view {
        Some(view) => {
            match dms
                .edit_message(&ctx, view.message_id, |m| {
                    m.embed(|e| display_user_view(&report, e))
                })
                .await
            {
                Ok(m) => m,
                Err(e) => {
                    log::error!("error sending message in dms for {:?}: {}", report, e);
                    return;
                }
            }
        }
        None => {
            match dms
                .send_message(&ctx, |m| m.embed(|e| display_user_view(&report, e)))
                .await
            {
                Ok(x) => x,
                Err(e) => {
                    log::error!("error sending message in dms for {:?}: {}", report, e);
                    return;
                }
            }
        }
    };

    let updated_model = UserViewModel {
        report_id: report.id,
        message_id: msg.id,
        status: report.status,
    };

    match db.save_user_view(updated_model).await {
        Ok(_) => {}
        Err(error) => {
            log::error!("error saving user view {:?} {:?}: {}", report, msg, error);
            return;
        }
    }
}

fn display_user_view<'a, 'b>(report: &ReportModel, e: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
    e.title(format!("Report (ID #{})", report.id))
        .field(
            "Provided Reason",
            report.reason.as_deref().unwrap_or("No reason provided!"),
            false,
        )
        .field("Location", "TOOD: implement this", false)
        .field("Status", format!("{:?}", report.status), false)
}

async fn update_mod_view(ctx: &Context, db: &Database, report: &ReportModel) {
    let view = match db.load_mod_view(report.id).await {
        Ok(v) => v,
        Err(error) => {
            log::error!("unable to load user view model {}: {}", report.id, error);
            return;
        }
    };

    let maybe_config = match db.maybe_load_server_config(report.guild_id).await {
        Ok(x) => x,
        Err(error) => {
            log::error!(
                "unable to load server config {}: {}",
                report.guild_id,
                error
            );
            return;
        }
    };

    // TODO: inform guild administrator? prevent report from being sent in?
    let config = match maybe_config {
        Some(x) => x,
        None => {
            log::error!("no server config found for {}", report.guild_id);
            return;
        }
    };

    // TODO: handle moving reports and whatnot?
    let channel_id = ChannelId(config.reports_channel);

    let reporter = match report.accuser_user_id.to_user(&ctx).await {
        Ok(x) => x,
        Err(e) => {
            log::error!("unable to fetch user {}: {}", report.accuser_user_id, e);
            return;
        }
    };

    let reported = match report.reported_user_id.to_user(&ctx).await {
        Ok(x) => x,
        Err(e) => {
            log::error!("unable to fetch user {}: {}", report.reported_user_id, e);
            return;
        }
    };

    let msg = match view {
        Some(view) => {
            match channel_id
                .edit_message(&ctx, view.message_id, |m| {
                    m.embed(|e| display_mod_view(&report, e, reported, reporter))
                })
                .await
            {
                Ok(m) => m,
                Err(e) => {
                    log::error!("error sending message in dms for {:?}: {}", report, e);
                    return;
                }
            }
        }
        None => {
            match channel_id
                .send_message(&ctx, |m| {
                    m.embed(|e| display_mod_view(&report, e, reported, reporter))
                })
                .await
            {
                Ok(x) => x,
                Err(e) => {
                    log::error!("error sending message in dms for {:?}: {}", report, e);
                    return;
                }
            }
        }
    };

    let updated_model = ModViewModel {
        report_id: report.id,
        channel_id,
        message_id: msg.id,
        // TODO: handle valid message id in archive
        preview_archive_id: 0,
        // TODO: handle mods clicking buttons n stuff
        handler: None,
    };

    match db.save_mod_view(updated_model).await {
        Ok(_) => {}
        Err(error) => {
            log::error!("error saving user view {:?} {:?}: {}", report, msg, error);
            return;
        }
    }
}

fn display_mod_view<'a, 'b>(
    report: &ReportModel,
    e: &'a mut CreateEmbed,
    reported: User,
    reporter: User,
) -> &'a mut CreateEmbed {
    // TODO: gotta be a better way yuck
    let reported_message_url = report.message_id.and_then(|m| {
        report.channel_id.map(|c| {
            format!(
                "https://discord.com/channels/{}/{}/{}",
                report.guild_id, c, m
            )
        })
    });

    let reporter_user = reporter;
    let reported_user = reported;

    let reported_user_avatar_url = reported_user
        .avatar_url()
        .unwrap_or_else(|| reported_user.default_avatar_url());
    let reported_user_mention = reported_user.mention();
    let reporter_user_mention = reporter_user.mention();
    // TODO: get channel name
    let location_channel_name = "#TODO".to_owned();

    let mut preview = String::new();
    if preview.trim().len() == 0 {
        preview.push_str("No preview available");
    }

    e.author(|a| {
        a.icon_url(reported_user_avatar_url)
            .name(reported_user.name)
    })
    .field("Reported User", reported_user_mention, true)
    .field("Status", "Unclaimed", true)
    .field("Reported By", reporter_user_mention, false)
    .field("Preview", preview, false);

    if let Some(location_link) = reported_message_url {
        e.field(
            "Location",
            format!("[{}]({})", location_channel_name, location_link),
            true,
        );
    }

    e
}
