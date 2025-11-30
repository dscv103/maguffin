//! Local cache module using SQLite.
//!
//! This module provides persistent local storage for PR data, stack metadata,
//! PR templates, and other cached information.

use crate::domain::template::PrTemplate;
use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

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

            CREATE TABLE IF NOT EXISTS pr_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                body TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Save a recent repository.
    pub fn save_recent_repository(&self, path: &str, owner: &str, name: &str) -> Result<()> {
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

    // ========================================================================
    // PR Template methods
    // ========================================================================

    /// Save a PR template.
    pub fn save_template(&self, template: &PrTemplate) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        // If this template is being set as default, unset any existing default
        if template.is_default {
            conn.execute("UPDATE pr_templates SET is_default = 0", [])
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        conn.execute(
            "INSERT OR REPLACE INTO pr_templates (id, name, body, is_default, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                template.id.to_string(),
                template.name,
                template.body,
                if template.is_default { 1 } else { 0 },
                template.created_at.to_rfc3339(),
                template.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get all PR templates.
    pub fn get_templates(&self) -> Result<Vec<PrTemplate>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let mut stmt = conn
            .prepare("SELECT id, name, body, is_default, created_at, updated_at FROM pr_templates ORDER BY name")
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let templates = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let body: String = row.get(2)?;
                let is_default: i32 = row.get(3)?;
                let created_at_str: String = row.get(4)?;
                let updated_at_str: String = row.get(5)?;

                let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(PrTemplate {
                    id,
                    name,
                    body,
                    is_default: is_default != 0,
                    created_at,
                    updated_at,
                })
            })
            .map_err(|e| StorageError::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(templates)
    }

    /// Get a specific template by ID.
    pub fn get_template(&self, id: &Uuid) -> Result<Option<PrTemplate>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let result = conn.query_row(
            "SELECT id, name, body, is_default, created_at, updated_at FROM pr_templates WHERE id = ?1",
            params![id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let body: String = row.get(2)?;
                let is_default: i32 = row.get(3)?;
                let created_at_str: String = row.get(4)?;
                let updated_at_str: String = row.get(5)?;

                let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(PrTemplate {
                    id,
                    name,
                    body,
                    is_default: is_default != 0,
                    created_at,
                    updated_at,
                })
            },
        );

        match result {
            Ok(template) => Ok(Some(template)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e.to_string()).into()),
        }
    }

    /// Get the default template, if one exists.
    pub fn get_default_template(&self) -> Result<Option<PrTemplate>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let result = conn.query_row(
            "SELECT id, name, body, is_default, created_at, updated_at FROM pr_templates WHERE is_default = 1 LIMIT 1",
            [],
            |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let body: String = row.get(2)?;
                let is_default: i32 = row.get(3)?;
                let created_at_str: String = row.get(4)?;
                let updated_at_str: String = row.get(5)?;

                let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(PrTemplate {
                    id,
                    name,
                    body,
                    is_default: is_default != 0,
                    created_at,
                    updated_at,
                })
            },
        );

        match result {
            Ok(template) => Ok(Some(template)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e.to_string()).into()),
        }
    }

    /// Delete a template by ID.
    pub fn delete_template(&self, id: &Uuid) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(format!("Lock error: {}", e)))?;

        let rows_affected = conn
            .execute("DELETE FROM pr_templates WHERE id = ?1", params![id.to_string()])
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(rows_affected > 0)
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

    #[test]
    fn test_save_and_get_template() {
        let cache = Cache::in_memory().unwrap();
        let template = PrTemplate::new("Test Template".to_string(), "Test body".to_string());

        cache.save_template(&template).unwrap();

        let retrieved = cache.get_template(&template.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Template");
        assert_eq!(retrieved.body, "Test body");
    }

    #[test]
    fn test_get_templates() {
        let cache = Cache::in_memory().unwrap();

        let template1 = PrTemplate::new("Alpha".to_string(), "Body 1".to_string());
        let template2 = PrTemplate::new("Beta".to_string(), "Body 2".to_string());

        cache.save_template(&template1).unwrap();
        cache.save_template(&template2).unwrap();

        let templates = cache.get_templates().unwrap();
        assert_eq!(templates.len(), 2);
        // Should be sorted by name
        assert_eq!(templates[0].name, "Alpha");
        assert_eq!(templates[1].name, "Beta");
    }

    #[test]
    fn test_default_template() {
        let cache = Cache::in_memory().unwrap();

        let template1 = PrTemplate::new("First".to_string(), "Body 1".to_string());
        let template2 =
            PrTemplate::new("Second".to_string(), "Body 2".to_string()).set_default(true);

        cache.save_template(&template1).unwrap();
        cache.save_template(&template2).unwrap();

        let default = cache.get_default_template().unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().name, "Second");
    }

    #[test]
    fn test_only_one_default_template() {
        let cache = Cache::in_memory().unwrap();

        let template1 =
            PrTemplate::new("First".to_string(), "Body 1".to_string()).set_default(true);
        let template2 =
            PrTemplate::new("Second".to_string(), "Body 2".to_string()).set_default(true);

        cache.save_template(&template1).unwrap();
        cache.save_template(&template2).unwrap();

        let templates = cache.get_templates().unwrap();
        let default_count = templates.iter().filter(|t| t.is_default).count();
        assert_eq!(default_count, 1);

        let default = cache.get_default_template().unwrap();
        assert_eq!(default.unwrap().name, "Second");
    }

    #[test]
    fn test_delete_template() {
        let cache = Cache::in_memory().unwrap();
        let template = PrTemplate::new("To Delete".to_string(), "Body".to_string());

        cache.save_template(&template).unwrap();
        assert!(cache.get_template(&template.id).unwrap().is_some());

        let deleted = cache.delete_template(&template.id).unwrap();
        assert!(deleted);
        assert!(cache.get_template(&template.id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_template() {
        let cache = Cache::in_memory().unwrap();
        let fake_id = Uuid::new_v4();

        let deleted = cache.delete_template(&fake_id).unwrap();
        assert!(!deleted);
    }
}
