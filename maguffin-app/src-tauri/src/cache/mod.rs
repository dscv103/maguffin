//! Local cache module using SQLite.
//!
//! This module provides persistent local storage for PR data, stack metadata,
//! and other cached information.

use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

/// Recent repository entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRepository {
    pub path: String,
    pub owner: String,
    pub name: String,
    pub last_opened: DateTime<Utc>,
}

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

    /// Save a recent repository.
    pub fn save_recent_repository(
        &self,
        path: &str,
        owner: &str,
        name: &str,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO repositories (path, owner, name, last_opened) VALUES (?1, ?2, ?3, ?4)",
            params![path, owner, name, now],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get recent repositories, ordered by last opened (most recent first).
    pub fn get_recent_repositories(&self, limit: usize) -> Result<Vec<RecentRepository>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT path, owner, name, last_opened FROM repositories ORDER BY last_opened DESC LIMIT ?1",
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let repos = stmt
            .query_map(params![limit as i64], |row| {
                let path: String = row.get(0)?;
                let owner: String = row.get(1)?;
                let name: String = row.get(2)?;
                let last_opened_str: String = row.get(3)?;

                let last_opened = DateTime::parse_from_rfc3339(&last_opened_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(RecentRepository {
                    path,
                    owner,
                    name,
                    last_opened,
                })
            })
            .map_err(|e| StorageError::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(repos)
    }

    /// Remove a repository from the recent list.
    pub fn remove_recent_repository(&self, path: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        conn.execute("DELETE FROM repositories WHERE path = ?1", params![path])
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

    #[test]
    fn test_recent_repositories() {
        let cache = Cache::in_memory().unwrap();

        cache
            .save_recent_repository("/path/to/repo1", "owner1", "repo1")
            .unwrap();
        cache
            .save_recent_repository("/path/to/repo2", "owner2", "repo2")
            .unwrap();

        let repos = cache.get_recent_repositories(10).unwrap();
        assert_eq!(repos.len(), 2);
        // Most recent first
        assert_eq!(repos[0].name, "repo2");
        assert_eq!(repos[1].name, "repo1");
    }

    #[test]
    fn test_remove_recent_repository() {
        let cache = Cache::in_memory().unwrap();

        cache
            .save_recent_repository("/path/to/repo1", "owner1", "repo1")
            .unwrap();
        cache.remove_recent_repository("/path/to/repo1").unwrap();

        let repos = cache.get_recent_repositories(10).unwrap();
        assert_eq!(repos.len(), 0);
    }
}
