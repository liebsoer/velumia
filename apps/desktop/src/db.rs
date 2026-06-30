use chrono::Utc;
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

pub const MIGRATION_VERSION: i64 = 2;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

pub struct AppDatabase {
    pub conn: Connection,
    pub data_dir: PathBuf,
}

impl AppDatabase {
    pub fn open(app_data_dir: &Path) -> Result<Self, DbError> {
        let db_dir = app_data_dir.join("db");
        fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("velumia.sqlite");
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self {
            conn,
            data_dir: app_data_dir.to_path_buf(),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), DbError> {
        let mut version: i64 = self
            .conn
            .query_row(
                "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if version < 1 {
            let sql = include_str!("../migrations/001_initial_schema.sql");
            self.conn.execute_batch(sql)?;
            self.record_migration(1)?;
            version = 1;
        }

        if version < 2 {
            let sql = include_str!("../migrations/002_prompt_content_syntax.sql");
            self.conn.execute_batch(sql)?;
            self.record_migration(2)?;
        }

        Ok(())
    }

    fn record_migration(&self, version: i64) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, now],
        )?;
        Ok(())
    }

    pub fn bootstrap_owner(&self, display_name: Option<&str>) -> Result<String, DbError> {
        if let Ok(id) = self.solo_user_id() {
            return Ok(id);
        }

        let user_id = Uuid::new_v4().to_string();
        let role_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO users (id, display_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            params![user_id, display_name, now],
        )?;
        tx.execute(
            "INSERT INTO roles (id, name, created_at) VALUES (?1, 'owner', ?2)",
            params![role_id, now],
        )?;

        for perm in OWNER_PERMISSIONS {
            tx.execute(
                "INSERT INTO role_permissions (role_id, permission) VALUES (?1, ?2)",
                params![role_id, perm],
            )?;
        }

        tx.execute(
            "INSERT INTO user_role_assignments (user_id, role_id, scope_type, scope_id) VALUES (?1, ?2, 'global', NULL)",
            params![user_id, role_id],
        )?;

        tx.commit()?;
        Ok(user_id)
    }

    pub fn solo_user_id(&self) -> Result<String, DbError> {
        self.conn
            .query_row("SELECT id FROM users LIMIT 1", [], |r| r.get(0))
            .map_err(DbError::from)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let result = self.conn.query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_setting(&self, key: &str, value_json: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            params![key, value_json],
        )?;
        Ok(())
    }

    pub fn is_first_launch(&self) -> Result<bool, DbError> {
        Ok(self.get_setting("wizard_completed")?.is_none())
    }
}

const OWNER_PERMISSIONS: &[&str] = &[
    "prompt:read",
    "prompt:write",
    "prompt:execute",
    "agent:read",
    "agent:write",
    "agent:execute",
    "skill:read",
    "skill:write",
    "skill:export",
    "library:import",
    "library:export",
    "credential:write",
    "settings:read",
    "settings:write",
    "admin",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn migration_and_bootstrap() {
        let dir = temp_dir().join(format!("velumia-test-{}", Uuid::new_v4()));
        let db = AppDatabase::open(&dir).expect("open");
        let user_id = db.bootstrap_owner(Some("Owner")).expect("bootstrap");
        assert!(!user_id.is_empty());
        assert_eq!(db.solo_user_id().unwrap(), user_id);
    }
}
