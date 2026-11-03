-- Records of migration runs
CREATE TABLE IF NOT EXISTS migration_runs (
    id              TEXT PRIMARY KEY,
    source_browser  TEXT NOT NULL,
    source_profile  TEXT NOT NULL,
    dest_browser    TEXT NOT NULL,
    dest_profile    TEXT NOT NULL,
    data_types      TEXT NOT NULL,  -- JSON array of DataCategory
    status          TEXT NOT NULL,  -- pending | running | completed | failed | cancelled
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    items_migrated  INTEGER NOT NULL DEFAULT 0,
    items_failed    INTEGER NOT NULL DEFAULT 0,
    error_message   TEXT
);

-- Per-item migration results for the report
CREATE TABLE IF NOT EXISTS migration_items (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES migration_runs(id) ON DELETE CASCADE,
    category        TEXT NOT NULL,
    item_key        TEXT NOT NULL,
    item_label      TEXT NOT NULL,
    status          TEXT NOT NULL,  -- migrated | failed | skipped | incompatible
    message         TEXT,
    migrated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_migration_items_run ON migration_items(run_id);
