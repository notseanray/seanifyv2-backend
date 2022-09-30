-- Add migration script here
CREATE TABLE IF NOT EXISTS songs
(
    id              TEXT PRIMARY KEY NOT NULL,
    title           TEXT             NOT NULL,
    artist          TEXT             NOT NULL,
    album           TEXT             NOT NULL,
    duration        FLOAT            NOT NULL,
    year            INTEGER          NOT NULL DEFAULT 0,
    genre           TEXT             NOT NULL,
    added_by        TEXT             NOT NULL,
    default_search  TEXT             NOT NULL
);

