use serenity::{
    client::Context,
    model::id::{ChannelId, MessageId, UserId},
};

use thiserror::Error;

pub async fn handle_err_dms<E: std::fmt::Display, M: ToString>(
    ctx: &Context,
    user_id: UserId,
    problematic_msg: Option<MessageId>,
    error: &E,
    msg: M,
) {
    log::warn!("informing user '{}' about error '{}'", user_id, error);

    // TODO: --> clean this up -->
    // try to inform the user about the error (and if we can't, log
    // this occurance)
    let dms = match user_id.create_dm_channel(&ctx).await {
        Ok(dms) => dms,
        Err(e) => {
            log::error!(
                "couldn't inform user about error '{}' because '{}'",
                error,
                e
            );
            return;
        }
    };

    match internal_handle_err(&ctx, dms.id, problematic_msg, error, msg).await {
        Ok(_) => {}
        Err(e) => {
            log::error!(
                "couldn't inform user '{}' about error '{}' because '{}'",
                user_id,
                error,
                e
            );
        }
    }
}

#[derive(Debug, Error)]
enum HandleErrError {
    #[error("Couldn't send error in public channel <#{0}>: {1}.\nInner error: {2}")]
    CouldntSend(ChannelId, serenity::Error, String),
}

pub async fn handle_err<E: std::fmt::Display, M: ToString>(
    ctx: &Context,
    channel_id: ChannelId,
    problematic_msg: Option<MessageId>,
    error: &E,
    msg: M,
) {
    // first, try to send error in a public channel - otherwise, try DM the user
    match internal_handle_err(&ctx, channel_id, problematic_msg, &error, msg.to_string()).await {
        Ok(_) => {}
        Err(e) => {
            if let Some(msg_id) = problematic_msg {
                if let Ok(msg2) = channel_id.message(&ctx, msg_id).await {
                    handle_err_dms(
                        &ctx,
                        msg2.author.id,
                        // don't include the problematic message as we cannot reference that in DMs
                        None,
                        &HandleErrError::CouldntSend(channel_id, e, format!("{}", error)),
                        msg,
                    )
                    .await;
                }
            }
        }
    };
}

async fn internal_handle_err<E: std::fmt::Display, M: ToString>(
    ctx: &Context,
    channel_id: ChannelId,
    problematic_msg: Option<MessageId>,
    error: &E,
    msg: M,
) -> Result<(), serenity::Error> {
    log::warn!(
        "informing about error '{}' in channel '{}'",
        error,
        channel_id
    );

    // TODO: extract this into function
    channel_id
        .send_message(&ctx, |m| {
            if let Some(msg) = problematic_msg {
                m.reference_message((channel_id, msg));
            }

            m.embed(|e| {
                e.title("Bot Error").field("Uh-oh!", msg, false).field(
                    "Error",
                    format!("{}", error),
                    false,
                )
            })
        })
        .await?;

    Ok(())
}
