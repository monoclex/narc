use serenity::model::id::*;

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

impl Into<i64> for ReportStatus {
    fn into(self) -> i64 {
        match self {
            Self::Unhandled => 0,
            Self::Reviewing => 1,
            Self::Accepted => 2,
            Self::Denied => 3,
        }
    }
}
