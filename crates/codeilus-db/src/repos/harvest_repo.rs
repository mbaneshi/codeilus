use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestRepoRow {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stars_today: Option<i64>,
    pub total_stars: Option<i64>,
    pub url: String,
    pub fingerprint: String,
    pub status: String,
    pub harvested_at: String,
}

pub struct HarvestRepoRepo {
    db: Arc<DbPool>,
}

impl HarvestRepoRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    pub fn insert(&self, repo: &HarvestRepoRow) -> CodeilusResult<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO harvested_repos (owner, name, description, language, stars_today, url, status, harvested_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                repo.owner,
                repo.name,
                repo.description,
                repo.language,
                repo.stars_today,
                repo.url,
                repo.status,
                repo.harvested_at,
            ],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_batch(&self, repos: &[HarvestRepoRow]) -> CodeilusResult<Vec<i64>> {
        let conn = self.db.connection();
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(repos.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO harvested_repos (owner, name, description, language, stars_today, url, status, harvested_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for repo in repos {
                stmt.execute(params![
                    repo.owner,
                    repo.name,
                    repo.description,
                    repo.language,
                    repo.stars_today,
                    repo.url,
                    repo.status,
                    repo.harvested_at,
                ])
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    pub fn get_by_fingerprint(&self, fingerprint: &str) -> CodeilusResult<Option<HarvestRepoRow>> {
        // Note: the harvested_repos table doesn't have a fingerprint column in the migration,
        // so fingerprints are managed in-memory. For DB persistence, we use owner+name matching.
        // This is a compatibility shim — a future migration should add a fingerprint column.
        let _ = fingerprint;
        Ok(None)
    }

    pub fn get_by_name(
        &self,
        owner: &str,
        name: &str,
    ) -> CodeilusResult<Option<HarvestRepoRow>> {
        let conn = self.db.connection();
        let result = conn.query_row(
            "SELECT id, owner, name, description, language, stars_today, url, status, harvested_date FROM harvested_repos WHERE owner = ?1 AND name = ?2 ORDER BY id DESC LIMIT 1",
            params![owner, name],
            |row| {
                Ok(HarvestRepoRow {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    language: row.get(4)?,
                    stars_today: row.get(5)?,
                    total_stars: None,
                    url: row.get(6)?,
                    fingerprint: String::new(),
                    status: row.get(7)?,
                    harvested_at: row.get(8)?,
                })
            },
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CodeilusError::Database(Box::new(e))),
        }
    }

    pub fn list(&self) -> CodeilusResult<Vec<HarvestRepoRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, owner, name, description, language, stars_today, url, status, harvested_date FROM harvested_repos ORDER BY id")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HarvestRepoRow {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    language: row.get(4)?,
                    stars_today: row.get(5)?,
                    total_stars: None,
                    url: row.get(6)?,
                    fingerprint: String::new(),
                    status: row.get(7)?,
                    harvested_at: row.get(8)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_by_status(&self, status: &str) -> CodeilusResult<Vec<HarvestRepoRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, owner, name, description, language, stars_today, url, status, harvested_date FROM harvested_repos WHERE status = ?1 ORDER BY id")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![status], |row| {
                Ok(HarvestRepoRow {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    language: row.get(4)?,
                    stars_today: row.get(5)?,
                    total_stars: None,
                    url: row.get(6)?,
                    fingerprint: String::new(),
                    status: row.get(7)?,
                    harvested_at: row.get(8)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_by_date(&self, date: &str) -> CodeilusResult<Vec<HarvestRepoRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, owner, name, description, language, stars_today, url, status, harvested_date FROM harvested_repos WHERE harvested_date = ?1 ORDER BY id")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![date], |row| {
                Ok(HarvestRepoRow {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    language: row.get(4)?,
                    stars_today: row.get(5)?,
                    total_stars: None,
                    url: row.get(6)?,
                    fingerprint: String::new(),
                    status: row.get(7)?,
                    harvested_at: row.get(8)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn update_status(&self, id: i64, status: &str) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute(
            "UPDATE harvested_repos SET status = ?1 WHERE id = ?2",
            params![status, id],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM harvested_repos", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
