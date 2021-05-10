-- We record every server we have welcomed. While we could use the `is_new`
-- parameter in `guild_create`, this will not catch servers we haven't welcomed
-- if the bot has some downtime.
--
-- We use this table to record every server we've ever welcomed, and upon
-- startup remove entries of servers we are no longer in to reduce the amount
-- of duplicate welcoming.
CREATE TABLE welcomed_servers (
    guild_id BIGINT PRIMARY KEY NOT NULL,

    welcomed DATETIME NOT NULL
);
