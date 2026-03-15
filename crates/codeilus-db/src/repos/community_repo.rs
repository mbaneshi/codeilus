use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::{CommunityId, SymbolId};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRow {
    pub id: CommunityId,
    pub label: String,
    pub cohesion: f64,
}

pub struct CommunityRepo {
    conn: Arc<Mutex<Connection>>,
}

impl CommunityRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, label: &str, cohesion: f64) -> CodeilusResult<CommunityId> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO communities (name, description, cohesion_score) VALUES (?1, '', ?2)",
            params![label, cohesion],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(CommunityId(conn.last_insert_rowid()))
    }

    pub fn insert_batch(
        &self,
        communities: &[(String, f64)],
    ) -> CodeilusResult<Vec<CommunityId>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(communities.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO communities (name, description, cohesion_score) VALUES (?1, '', ?2)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (label, cohesion) in communities {
                stmt.execute(params![label, cohesion])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(CommunityId(tx.last_insert_rowid()));
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    pub fn insert_member(
        &self,
        community_id: CommunityId,
        symbol_id: SymbolId,
    ) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO community_members (community_id, symbol_id) VALUES (?1, ?2)",
            params![community_id.0, symbol_id.0],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn insert_members_batch(
        &self,
        members: &[(CommunityId, SymbolId)],
    ) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        {
            let mut stmt = tx
                .prepare(
                    "INSERT OR IGNORE INTO community_members (community_id, symbol_id) VALUES (?1, ?2)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (community_id, symbol_id) in members {
                stmt.execute(params![community_id.0, symbol_id.0])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn get(&self, id: CommunityId) -> CodeilusResult<CommunityRow> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.query_row(
            "SELECT id, name, cohesion_score FROM communities WHERE id = ?1",
            params![id.0],
            |row| {
                Ok(CommunityRow {
                    id: CommunityId(row.get(0)?),
                    label: row.get(1)?,
                    cohesion: row.get(2)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CodeilusError::NotFound(format!("Community {id} not found"))
            }
            _ => CodeilusError::Database(Box::new(e)),
        })
    }

    pub fn list(&self) -> CodeilusResult<Vec<CommunityRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT id, name, cohesion_score FROM communities")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CommunityRow {
                    id: CommunityId(row.get(0)?),
                    label: row.get(1)?,
                    cohesion: row.get(2)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_members(&self, community_id: CommunityId) -> CodeilusResult<Vec<SymbolId>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT symbol_id FROM community_members WHERE community_id = ?1")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![community_id.0], |row| {
                Ok(SymbolId(row.get(0)?))
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Find which community a symbol belongs to.
    pub fn find_by_symbol(&self, symbol_id: SymbolId) -> CodeilusResult<Option<CommunityRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let result = conn.query_row(
            "SELECT c.id, c.name, c.cohesion_score FROM communities c \
             JOIN community_members cm ON cm.community_id = c.id \
             WHERE cm.symbol_id = ?1",
            params![symbol_id.0],
            |row| {
                Ok(CommunityRow {
                    id: CommunityId(row.get(0)?),
                    label: row.get(1)?,
                    cohesion: row.get(2)?,
                })
            },
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CodeilusError::Database(Box::new(e))),
        }
    }

    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM community_members", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute("DELETE FROM communities", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
