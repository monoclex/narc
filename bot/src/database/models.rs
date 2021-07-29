use serenity::{
    model::{channel::ReactionType, id::*},
    utils::Colour,
};
use thiserror::Error;

#[derive(Clone)]
pub struct ServerConfiguration {
    pub emoji_builtin: Option<String>,
    pub emoji_custom: Option<u64>,
    pub prefix: Option<String>,
    pub reports_channel: u64,
}

impl ServerConfiguration {
    pub fn matches_emoji(&self, emoji: &ReactionType) -> bool {
        match (self.emoji_builtin.as_ref(), self.emoji_custom, emoji) {
            (_, Some(custom), ReactionType::Custom { id, .. }) => custom == id.0,
            (Some(builtin), _, ReactionType::Unicode(unicode)) => builtin == unicode,
            (None, None, ReactionType::Unicode(unicode)) => unicode == "🚩",
            _ => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum ChannelLookupError {
    #[error("Report has no channel!")]
    NoReportChannel,
    #[error("Non GuildChannel: {0}")]
    NonGuildChannel(String),
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),
}

#[derive(Debug, Clone)]
pub struct ReportModel {
    pub id: u64,
    pub accuser_user_id: UserId,
    pub reported_user_id: UserId,
    pub guild_id: GuildId,
    pub status: ReportStatus,
    pub channel_id: Option<ChannelId>,
    pub message_id: Option<MessageId>,
    pub reason: Option<String>,
}

impl ReportModel {
    pub fn url(&self) -> Option<String> {
        self.message_id.and_then(|m| {
            self.channel_id
                .map(|c| format!("https://discord.com/channels/{}/{}/{}", self.guild_id, c, m))
        })
    }

    pub async fn channel_name(
        &self,
        ctx: &serenity::client::Context,
    ) -> Result<String, ChannelLookupError> {
        fn channel_to_name(channel: serenity::model::channel::Channel) -> String {
            match channel {
                serenity::model::channel::Channel::Guild(c) => c.kind.name().to_string(),
                serenity::model::channel::Channel::Private(c) => c.kind.name().to_string(),
                serenity::model::channel::Channel::Category(c) => c.kind.name().to_string(),
                _ => "unknown channel type".to_string(),
            }
        }

        match self.channel_id {
            Some(c) => match c.name(&ctx).await {
                Some(name) => Ok(name),
                None => match ctx.http.get_channel(c.0).await? {
                    serenity::model::channel::Channel::Guild(channel) => Ok(channel.name),
                    channel => Err(ChannelLookupError::NonGuildChannel(channel_to_name(
                        channel,
                    ))),
                },
            },
            None => Err(ChannelLookupError::NoReportChannel),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewModel {
    User(UserViewModel),
    Mod(ModViewModel),
}

#[derive(Debug, Clone)]
pub struct UserViewModel {
    pub report_id: u64,
    pub message_id: MessageId,
    pub status: ReportStatus,
}

#[derive(Debug, Clone)]
pub struct ModViewModel {
    pub report_id: u64,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub preview_archive_id: u64,
    pub handler: Option<UserId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReportStatus {
    Unhandled,
    Reviewing,
    Accepted,
    Denied,
}

impl ReportStatus {
    pub fn into_human_status(self) -> &'static str {
        match self {
            ReportStatus::Unhandled => "😴 Unhandled",
            ReportStatus::Reviewing => "🔎 Reviewing",
            ReportStatus::Accepted => "✅ Accepted",
            ReportStatus::Denied => "❌ Denied",
        }
    }

    pub fn into_color(self) -> Option<Colour> {
        Some(Colour::new(match self {
            ReportStatus::Unhandled => return None,
            ReportStatus::Reviewing => 0xADD8E6,
            ReportStatus::Denied => 0xFF0000,
            ReportStatus::Accepted => 0x00FF00,
        }))
    }
}

impl From<i64> for ReportStatus {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Unhandled,
            1 => Self::Reviewing,
            2 => Self::Accepted,
            3 => Self::Denied,
            _ => panic!("unknown ReportStatus {}", value),
        }
    }
}

impl From<ReportStatus> for i64 {
    fn from(val: ReportStatus) -> Self {
        match val {
            ReportStatus::Unhandled => 0,
            ReportStatus::Reviewing => 1,
            ReportStatus::Accepted => 2,
            ReportStatus::Denied => 3,
        }
    }
}
