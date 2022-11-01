-- Add migration script here
-- TODO
CREATE TABLE IF NOT EXISTS playlist
(
    name            TEXT PRIMARY KEY NOT NULL,
    public_playlist BOOLEAN          NOT NULL DEFAULT TRUE,
    songs           TEXT             NOT NULL,
    author          TEXT             NOT NULL,
    description     TEXT             NOT NULL,
    likes           TEXT             NOT NULL,
    cover           TEXT             NOT NULL,
    duration        UNSIGNED INTEGER NOT NULL DEFAULT 0,
    lastupdate      TEXT             NOT NULL DEFAULT 0
);

