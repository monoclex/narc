use serenity::{
    client::Context,
    model::id::{ChannelId, MessageId, UserId},
};

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

    handle_err(&ctx, dms.id, problematic_msg, error, msg).await;
}

pub async fn handle_err<E: std::fmt::Display, M: ToString>(
    ctx: &Context,
    channel_id: ChannelId,
    problematic_msg: Option<MessageId>,
    error: &E,
    msg: M,
) {
    log::warn!(
        "informing about error '{}' in channel '{}'",
        error,
        channel_id
    );

    // TODO: extract this into function
    match channel_id
        .send_message(&ctx, |m| {
            if let Some(msg) = problematic_msg {
                m.reference_message((channel_id, msg));
            }

            m.embed(|e| {
                e.title("Bot Error").field("Uh-oh!", msg, false).field(
                    "Internal Error",
                    format!("{}", error),
                    false,
                )
            })
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            log::error!(
                "couldn't inform user about error '{}' because '{}'",
                error,
                e
            );
            return;
        }
    };
}
