-- Add migration script here
-- TODO
CREATE TABLE IF NOT EXISTS playlist
(
    name            TEXT PRIMARY KEY NOT NULL,
    public_playlist BOOLEAN          NOT NULL DEFAULT TRUE,
    songs           TEXT[]           NOT NULL,
    author          TEXT             NOT NULL,
    author_id       TEXT             NOT NULL,
    edit_list       TEXT[]           NOT NULL,
    description     TEXT             NOT NULL,
    likes           TEXT[]           NOT NULL,
    cover           TEXT             NOT NULL,
    duration        BIGINT           NOT NULL DEFAULT 0,
    lastupdate      TEXT             NOT NULL DEFAULT 0
);

