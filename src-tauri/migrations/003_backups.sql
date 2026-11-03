-- Backup manifest
CREATE TABLE IF NOT EXISTS backups (
    id          TEXT PRIMARY KEY,
    browser     TEXT NOT NULL,
    profile     TEXT NOT NULL,
    path        TEXT NOT NULL,
    size_bytes  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    encrypted   INTEGER NOT NULL DEFAULT 1  -- boolean
);
