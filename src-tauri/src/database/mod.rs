use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

/// Central application database
pub struct AppDatabase {
    conn: Connection,
}

impl AppDatabase {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;")?;

        let mut db = AppDatabase { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        let applied: Vec<i64> = {
            let mut stmt = self
                .conn
                .prepare("SELECT version FROM schema_migrations ORDER BY version")?;
            stmt.query_map([], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect()
        };

        let migrations = Self::migrations();
        for (version, sql) in &migrations {
            if !applied.contains(version) {
                self.conn.execute_batch(sql)?;
                self.conn.execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1)",
                    params![version],
                )?;
                log::info!("Applied database migration v{}", version);
            }
        }
        Ok(())
    }

    fn migrations() -> Vec<(i64, &'static str)> {
        vec![
            (1, include_str!("../../migrations/001_initial.sql")),
            (2, include_str!("../../migrations/002_migration_history.sql")),
            (3, include_str!("../../migrations/003_backups.sql")),
        ]
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Record a completed migration run
    pub fn record_migration(&self, record: &MigrationRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO migration_runs
             (id, source_browser, source_profile, dest_browser, dest_profile,
              data_types, status, started_at, completed_at, items_migrated,
              items_failed, error_message)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                record.id,
                record.source_browser,
                record.source_profile,
                record.dest_browser,
                record.dest_profile,
                record.data_types,
                record.status,
                record.started_at,
                record.completed_at,
                record.items_migrated,
                record.items_failed,
                record.error_message,
            ],
        )?;
        Ok(())
    }

   use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

/// Central application database
pub struct AppDatabase {
    conn: Connection,
}

impl AppDatabase {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;")?;

        let mut db = AppDatabase { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        let applied: Vec<i64> = {
            let mut stmt = self
                .conn
                .prepare("SELECT version FROM schema_migrations ORDER BY version")?;
            
            // Fixed E0597: Evaluate the query map safely by assigning it explicitly 
            // before breaking down the iterator chaining
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            
            rows.filter_map(|r| r.ok()).collect()
        };

        let migrations = Self::migrations();
        for (version, sql) in &migrations {
            if !applied.contains(version) {
                self.conn.execute_batch(sql)?;
                self.conn.execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1)",
                    params![version],
                )?;
                log::info!("Applied database migration v{}", version);
            }
        }
        Ok(())
    }

    fn migrations() -> Vec<(i64, &'static str)> {
        vec![
            (1, include_str!("../../migrations/001_initial.sql")),
            (2, include_str!("../../migrations/002_migration_history.sql")),
            (3, include_str!("../../migrations/003_backups.sql")),
        ]
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Record a completed migration run
    pub fn record_migration(&self, record: &MigrationRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO migration_runs
             (id, source_browser, source_profile, dest_browser, dest_profile,
              data_types, status, started_at, completed_at, items_migrated,
              items_failed, error_message)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                record.id,
                record.source_browser,
                record.source_profile,
                record.dest_browser,
                record.dest_profile,
                record.data_types,
                record.status,
                record.started_at,
                record.completed_at,
                record.items_migrated,
                record.items_failed,
                record.error_message,
            ],
        )?;
        Ok(())
    }
/// Persist the per-item results of a migration run so the report
    /// survives an app restart. Called once a run reaches a terminal state.
    pub fn record_migration_items(
        &self,
        run_id: &str,
        items: &[MigrationItemRecord],
    ) -> Result<()> {
        for item in items {
            self.conn.execute(
                "INSERT INTO migration_items
                 (id, run_id, category, item_key, item_label, status, message)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    run_id,
                    item.category,
                    item.item_key,
                    item.item_label,
                    item.status,
                    item.message,
                ],
            )?;
        }
        Ok(())
    }

    /// Retrieve the persisted per-item report for a migration run.
    /// Used when the in-memory status cache no longer has the run
    /// (e.g. after an app restart) but the run is in history.
    pub fn get_migration_items(&self, run_id: &str) -> Result<Vec<MigrationItemRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, item_key, item_label, status, message
             FROM migration_items
             WHERE run_id = ?1
             ORDER BY migrated_at",
        )?;
        let rows = stmt.query_map(params![run_id], |row| {
            Ok(MigrationItemRecord {
                category: row.get(0)?,
                item_key: row.get(1)?,
                item_label: row.get(2)?,
                status: row.get(3)?,
                message: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn list_migration_runs(&self, limit: u32) -> Result<Vec<MigrationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_browser, source_profile, dest_browser, dest_profile,
                    data_types, status, started_at, completed_at, items_migrated,
                    items_failed, error_message
             FROM migration_runs ORDER BY started_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(MigrationRecord {
                id: row.get(0)?,
                source_browser: row.get(1)?,
                source_profile: row.get(2)?,
                dest_browser: row.get(3)?,
                dest_profile: row.get(4)?,
                data_types: row.get(5)?,
                status: row.get(6)?,
                started_at: row.get(7)?,
                completed_at: row.get(8)?,
                items_migrated: row.get(9)?,
                items_failed: row.get(10)?,
                error_message: row.get(11)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Save and retrieve app settings (key-value)
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Backup metadata
    pub fn record_backup(&self, backup: &BackupRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO backups (id, browser, profile, path, size_bytes, created_at, encrypted)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                backup.id,
                backup.browser,
                backup.profile,
                backup.path,
                backup.size_bytes,
                backup.created_at,
                backup.encrypted,
            ],
        )?;
        Ok(())
    }

    pub fn list_backups(&self) -> Result<Vec<BackupRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, browser, profile, path, size_bytes, created_at, encrypted
             FROM backups ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BackupRecord {
                id: row.get(0)?,
                browser: row.get(1)?,
                profile: row.get(2)?,
                path: row.get(3)?,
                size_bytes: row.get(4)?,
                created_at: row.get(5)?,
                encrypted: row.get(6)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_backup(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM backups WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationItemRecord {
    pub category: String,
    pub item_key: String,
    pub item_label: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MigrationRecord {
    pub id: String,
    pub source_browser: String,
    pub source_profile: String,
    pub dest_browser: String,
    pub dest_profile: String,
    pub data_types: String, // JSON array
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub items_migrated: i64,
    pub items_failed: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub browser: String,
    pub profile: String,
    pub path: String,
    pub size_bytes: i64,
    pub created_at: String,
    pub encrypted: bool,
}
