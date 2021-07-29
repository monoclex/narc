use serenity::{
    client::Context,
    model::{
        guild::{Guild, Member},
        id::{GuildId, UserId},
        prelude::User,
    },
    utils::{content_safe, parse_username, ContentSafeOptions},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FailedUserParse {
    #[error("No user '{0}' found")]
    NoUser(String),
    #[error("Username too ambiguous, found alternative matches")]
    Ambiguous(Vec<(Member, String)>),
}

pub async fn user(
    argument: &str,
    ctx: &Context,
    guild: &Guild,
) -> Result<ParsedUser, FailedUserParse> {
    // mentioning the user
    if let Some(id) = parse_username(argument) {
        return Ok(id.into());
    }

    // pasting their user id
    if let Ok(id) = argument.parse::<u64>() {
        if let Ok(member) = guild.member(&ctx, id).await {
            return Ok(member.into());
        }
    }

    // by exact member name
    if let Some(member) = guild.member_named(argument) {
        return Ok(member.to_owned().into());
    }

    // by exact member nickname
    // TODO: check if this works
    if let Some((_, member)) = guild.members.iter().find(|(_, m)| {
        m.nick
            .as_ref()
            .map(|n| n.as_str() == argument)
            .unwrap_or(false)
    }) {
        return Ok(member.to_owned().into());
    }

    // by similar members
    // note: ensure that there is only one possibility to reduce user error
    let similar_members = guild.members_starting_with(argument, false, false).await;
    if similar_members.len() == 1 {
        let (member, _) = similar_members.into_iter().next().unwrap();
        return Ok(member.to_owned().into());
    }

    if similar_members.len() > 1 {
        // if too many similar members, complain about it
        return Err(FailedUserParse::Ambiguous(
            similar_members
                .into_iter()
                .map(|(l, r)| (l.to_owned(), r))
                .collect::<Vec<_>>(),
        ));
    }

    // by members with argument in their name
    let similar_members = guild.members_containing(argument, false, false).await;
    if similar_members.len() == 1 {
        let (member, _) = similar_members.into_iter().next().unwrap();
        return Ok(member.to_owned().into());
    }

    if similar_members.len() > 1 {
        // if too many similar members, complain about it
        return Err(FailedUserParse::Ambiguous(
            similar_members
                .into_iter()
                .map(|(l, r)| (l.to_owned(), r))
                .collect::<Vec<_>>(),
        ));
    }

    // TODO: maybe use more of the `guild.members_x` methods?

    let sanitized_username = content_safe(&ctx, argument, &ContentSafeOptions::default()).await;
    Err(FailedUserParse::NoUser(sanitized_username))
}

pub enum ParsedUser {
    Member(Member),
    Id(UserId),
}

impl ParsedUser {
    pub fn user_id(&self) -> UserId {
        match self {
            ParsedUser::Member(member) => member.user.id,
            ParsedUser::Id(id) => *id,
        }
    }

    pub async fn user(self, ctx: &Context) -> Option<User> {
        match self {
            ParsedUser::Member(member) => Some(member.user),
            ParsedUser::Id(id) => ctx.cache.user(id).await,
        }
    }

    pub async fn member(self, ctx: &Context, guild_id: GuildId) -> serenity::Result<Member> {
        match self {
            ParsedUser::Member(member) => Ok(member),
            ParsedUser::Id(id) => guild_id.member(&ctx, id).await,
        }
    }
}

impl From<u64> for ParsedUser {
    fn from(val: u64) -> Self {
        ParsedUser::Id(val.into())
    }
}

impl From<UserId> for ParsedUser {
    fn from(val: UserId) -> Self {
        ParsedUser::Id(val)
    }
}

impl From<Member> for ParsedUser {
    fn from(val: Member) -> Self {
        ParsedUser::Member(val)
    }
}
