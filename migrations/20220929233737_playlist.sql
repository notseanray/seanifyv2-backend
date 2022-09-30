-- Add migration script here
-- TODO
CREATE TABLE IF NOT EXISTS playlist
(
    id              TEXT PRIMARY KEY NOT NULL,
    title           TEXT             NOT NULL,
    album           TEXT                     ,
    artist          TEXT             NOT NULL,
    duration        FLOAT            NOT NULL,
    genre           TEXT                     ,
    album_artist    TEXT                     ,
    lastupdate      UNSIGNED INTEGER          NOT NULL DEFAULT 0
);

