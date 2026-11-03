-- App settings (key-value store)
CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Default settings
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('theme', '"dark"'),
    ('auto_backup', 'true'),
    ('backup_encryption', 'true'),
    ('consent_given', 'false'),
    ('max_history_days', '90');
