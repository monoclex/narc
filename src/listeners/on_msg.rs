//! Used to handle when Narc pins a message, and then prompty deletes it.

use serenity::{
    client::Context,
    model::{
        channel::{Message, MessageReference, MessageType},
        id::{ChannelId, MessageId},
    },
};
use thiserror::Error;

use crate::state::State;

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("A data race occurred when trying to delete the pinned message")]
    DataRace,
    #[error("The pinned message could not be detected")]
    NoPinReference,
    #[error("A Discord error occurred: {0}")]
    DiscordError(#[from] serenity::Error),
}

pub async fn message(ctx: &Context, message: &Message) -> Result<(), MessageError> {
    let read = ctx.data.read().await;
    let state = read.get::<State>().unwrap();

    if !matches!(message.kind, MessageType::PinsAdd) {
        return Ok(());
    }

    let has_message_marked_for_deletion = {
        let pinned = state.pinned_msgs.read().await;

        let (chan_id, msg_id) = message
            .message_reference
            .as_ref()
            .and_then(map)
            .ok_or(MessageError::NoPinReference)?;

        fn map(m: &MessageReference) -> Option<(ChannelId, MessageId)> {
            match m.message_id {
                Some(m_id) => Some((m.channel_id, m_id)),
                _ => None,
            }
        }

        pinned
            .iter()
            .any(|(chan, msg)| chan_id == *chan && msg_id == *msg)
    };

    if has_message_marked_for_deletion {
        let mut pinned = state.pinned_msgs.write().await;

        let (chan_id, msg_id) = message
            .message_reference
            .as_ref()
            .and_then(map)
            .ok_or(MessageError::NoPinReference)?;

        fn map(m: &MessageReference) -> Option<(ChannelId, MessageId)> {
            match m.message_id {
                Some(m_id) => Some((m.channel_id, m_id)),
                _ => None,
            }
        }

        let pinned_msg_index = pinned
            .iter()
            .position(|(chan, msg)| chan_id == *chan && msg_id == *msg)
            .ok_or(MessageError::DataRace)?;

        pinned.remove(pinned_msg_index);
        message.delete(&ctx).await?;
    }

    Ok(())
}
