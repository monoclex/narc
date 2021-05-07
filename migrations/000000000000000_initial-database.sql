-- --- --- --
-- STORAGE --
-- --- --- --

-- Table to hold bot configuration for the server
CREATE TABLE server_configuration (
    -- Each server has configuration based on its id
    guild_id BIGINT PRIMARY KEY NOT NULL,

    -- The duration of which the server has premium for, if at all
    premium_duration INTEGER,

    -- Each server has a channel to put reports into
    reports_channel BIGINT NOT NULL,

    -- An optional builtin twemoji (unicode) to use for reports
    emoji_builtin TEXT,

    -- An optional custom uploaded emoji to use for reports
    emoji_custom BIGINT,

    -- An optional prefix to take priority over `n!`
    prefix TEXT
);

-- Stores every report
CREATE TABLE reports (
    -- Each report gets a unique ID for moderators to reference/use
    id INTEGER PRIMARY KEY NOT NULL,

    -- The user submitting this report
    accuser_user_id BIGINT NOT NULL,

    -- The user that is being reported
    reported_user_id BIGINT NOT NULL,

    -- Each report is associated with a guild, where the message took place
    guild_id BIGINT NOT NULL,

    -- The current status of the report
    status INTEGER NOT NULL,

    -- Each report may reference a target message. If there is no ID for the,
    -- channel or message, then this report was created via a command or other
    -- means
    channel_id BIGINT,
    message_id BIGINT,

    -- Each report may have a reasoning behind it
    reason TEXT
);

-- Backups of individual messages so moderators can assess the situation as it
-- was reported at the time.
CREATE TABLE message_archive (
    -- A unique ID for the archived message, as we may need to store edits over
    -- time and thus cannot use the `message_id` as a key
    id INTEGER PRIMARY KEY NOT NULL,

    -- The ID of the message that is archived
    message_id BIGINT NOT NULL,

    -- The content of the text
    content TEXT NOT NULL
);

-- Enable fast searching by message id, since there may be multiple copies of a
-- given message in the archive.
CREATE INDEX message_archive_by_message_id
    ON message_archive(message_id, id ASC);

-- Messages may contain embeds. We store those in an alternative table as to
-- not affect the lookup of archived messages.
CREATE TABLE attachment_archive (
    -- The ID of the archived message. This will map to `message_archive.id`.
    id INTEGER NOT NULL,

    -- The index of this attachment, used for ordering the attachments by the
    -- order in which they were uploaded. Zero-based.
    idx INTEGER NOT NULL,

    -- The original URL that the attachment was uploaded to.
    url TEXT NOT NULL,

    -- The raw attachment. May be null if the user is not a premium user of the
    -- bot (as storing attachments for all users would be costly).
    attachment BLOB
);

-- Index archived attachments by their `id` so we can load them in conjunction
-- with archived messages in `message_archive`, and sort them by `idx` since we
-- will be fetching attachments in the order they were posted.
CREATE INDEX attachment_archive_by_id ON attachment_archive(id, idx ASC);

-- --- --- --- --- --- --- --
-- DISCORD SYNCHRONIZATION --
-- --- --- --- --- --- --- --

-- When a user submits a report, they receive a message from the bot to receive
-- status updates on how the report is being handled.
CREATE TABLE discord_user_view (
    -- The ID of the report this submission reflects
    report_id INTEGER PRIMARY KEY NOT NULL,

    -- The ID of the message (in DMs) that this report is located at
    message_id INTEGER NOT NULL,

    -- The last status of the report (used to figure out if we need to notify
    -- the user upon the report being solved)
    status INTEGER NOT NULL
);

-- When a user submits a report, moderators receive a message in the reports
-- channel detailing the report and information about it.
CREATE TABLE discord_mod_view (
    -- The ID of the report
    report_id INTEGER PRIMARY KEY NOT NULL,

    -- The channel the report is in (useful if the reports channel changes)
    channel_id INTEGER NOT NULL,

    -- The ID of the report view message
    message_id INTEGER NOT NULL,

    -- The ID of the archived message that's used in the preview of the report
    preview_archive_id INTEGER NOT NULL,

    -- The ID of the moderator handling/ed the report (none if no handler yet)
    handler INTEGER
);
