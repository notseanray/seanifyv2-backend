-- Add migration script here
CREATE TABLE IF NOT EXISTS songs
(
    id              TEXT PRIMARY KEY NOT NULL,
    title           TEXT             NOT NULL,
    uploader        TEXT             NOT NULL,
    artist          TEXT             NOT NULL,
    genre           TEXT             NOT NULL,
    album           TEXT             NOT NULL,
    duration        FLOAT            NOT NULL,
    age_limit       UNSIGNED INTEGER NOT NULL DEFAULT 0,
    webpage_url     TEXT             NOT NULL,
    was_live        BOOLEAN          NOT NULL DEFAULT FALSE,
    upload_date     TEXT             NOT NULL,
    filesize        INTEGER          NOT NULL,
    added_by        TEXT             NOT NULL,
    default_search  TEXT             NOT NULL
);

