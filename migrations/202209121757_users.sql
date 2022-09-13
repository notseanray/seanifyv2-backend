CREATE TABLE IF NOT EXISTS users
(
    id              TEXT PRIMARY KEY NOT NULL,
    username        TEXT             NOT NULL,
    serverside      BOOLEAN          NOT NULL DEFAULT TRUE,
    thumbnails      BOOLEAN          NOT NULL DEFAULT TRUE,
    autoplay        BOOLEAN          NOT NULL DEFAULT TRUE,
    allow_followers BOOLEAN          NOT NULL DEFAULT TRUE,
    public_account  BOOLEAN          NOT NULL DEFAULT TRUE,
    activity        BOOLEAN          NOT NULL DEFAULT TRUE,
    last_played     TEXT             NOT NULL,
    display_name    TEXT             NOT NULL,
    followers       TEXT             NOT NULL,
    following       TEXT             NOT NULL,
    analytics       BOOLEAN          NOT NULL
);
