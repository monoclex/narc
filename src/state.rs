use serenity::{
    model::{
        channel::Message,
        id::{ChannelId, MessageId, UserId},
    },
    prelude::*,
};
use std::collections::HashMap;

/// State is used to store small bits of information about users, to make the
/// bot feel nicer to use.
///
/// For example, it handles when a user is in `n!setup` to prevent triggering
/// reports.
pub struct State {
    // TODO: use async concurrent hashmap
    pub users: RwLock<HashMap<UserId, UserState>>,
    pub pinned_msgs: RwLock<Vec<(ChannelId, MessageId)>>,
}

impl TypeMapKey for State {
    type Value = State;
}

impl State {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            pinned_msgs: RwLock::new(Vec::new()),
        }
    }

    pub async fn pin_msg(&self, msg: &Message, ctx: &Context) -> serenity::Result<()> {
        let mut pinned = self.pinned_msgs.write().await;
        pinned.push((msg.channel_id.clone(), msg.id.clone()));
        msg.pin(ctx).await
    }

    pub async fn get_user(&self, id: &UserId) -> UserState {
        let lock = self.users.read().await;
        match lock.get(&id) {
            None => Default::default(),
            Some(user_state) => user_state.clone(),
        }
    }

    pub async fn mutate_user<F>(&self, id: &UserId, mutation: F)
    where
        F: FnOnce(&mut UserState),
    {
        let mut lock = self.users.write().await;
        let element = lock.entry(*id).or_default();
        mutation(element)
    }
}

#[derive(Clone)]
pub struct UserState {
    pub in_setup: bool,
}

impl Default for UserState {
    fn default() -> Self {
        Self { in_setup: false }
    }
}

impl UserState {
    pub fn can_make_report(&self) -> bool {
        !self.in_setup
    }
}
