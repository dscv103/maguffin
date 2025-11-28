//! Local cache module using SQLite.
//!
//! This module provides persistent local storage for PR data, stack metadata,
//! and other cached information.

use crate::error::{Result, StorageError};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

/// Local SQLite cache for the application.
pub struct Cache {
    conn: Mutex<Connection>,
}

impl Cache {
    /// Open or create a cache at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| StorageError::Database(e.to_string()))?;

        let cache = Self {
            conn: Mutex::new(conn),
        };

        cache.initialize()?;
        Ok(cache)
    }

    /// Open an in-memory cache (useful for testing).
    pub fn in_memory() -> Result<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| StorageError::Database(e.to_string()))?;

        let cache = Self {
            conn: Mutex::new(conn),
        };

        cache.initialize()?;
        Ok(cache)
    }

    /// Initialize the database schema.
    fn initialize(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS repositories (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                last_opened TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pull_requests (
                id INTEGER PRIMARY KEY,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                data TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (repo_id) REFERENCES repositories(id),
                UNIQUE(repo_id, number)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Save a setting.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a setting.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e.to_string()).into()),
        }
    }

    /// Clear all cached data.
    pub fn clear(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        conn.execute_batch(
            r#"
            DELETE FROM pull_requests;
            DELETE FROM repositories;
            "#,
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_in_memory() {
        let cache = Cache::in_memory().unwrap();
        assert!(cache.get_setting("test").unwrap().is_none());
    }

    #[test]
    fn test_cache_set_get_setting() {
        let cache = Cache::in_memory().unwrap();
        cache.set_setting("test_key", "test_value").unwrap();

        let value = cache.get_setting("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[test]
    fn test_cache_overwrite_setting() {
        let cache = Cache::in_memory().unwrap();
        cache.set_setting("key", "value1").unwrap();
        cache.set_setting("key", "value2").unwrap();

        let value = cache.get_setting("key").unwrap();
        assert_eq!(value, Some("value2".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::in_memory().unwrap();
        cache.set_setting("key", "value").unwrap();
        cache.clear().unwrap();

        // Settings are not cleared, only data tables
        let value = cache.get_setting("key").unwrap();
        assert_eq!(value, Some("value".to_string()));
    }
}
