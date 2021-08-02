-- We record a list of protected users - these are users that will not have
-- reports be triggered for them.
CREATE TABLE protected_users (
    guild_id BIGINT NOT NULL,
    protected_user_id BIGINT NOT NULL,

    PRIMARY KEY (guild_id, protected_user_id)
);
