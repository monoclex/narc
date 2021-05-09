use std::collections::HashMap;

use serenity::{model::id::GuildId, prelude::RwLock};

use super::Database;

mod get_server_prefix;

pub struct Cache {
    // TODO: find hashmap that clears old machines
    server_prefixes: RwLock<HashMap<GuildId, Option<String>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            server_prefixes: RwLock::new(HashMap::new()),
        }
    }
}

// wiping stuff
impl Cache {
    pub async fn wipe_server_prefix_cache(&self, server: &GuildId) {
        let mut lock = self.server_prefixes.write().await;
        lock.remove(server);
    }
}

// helpers
impl Database {
    pub async fn has_server_config(&self, guild_id: &GuildId) -> Result<bool, sqlx::Error> {
        self.get_server_prefix(guild_id).await.map(|p| p.is_some())
    }
}
