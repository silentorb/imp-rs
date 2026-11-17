//! SQLite derived projection store (Miro profile).

use crate::signal::{ChangeKind, SyncScope, SyncSignal};
use crate::store::{Store, StoreCapabilities, StoreError, StoreRecord};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS records (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  content TEXT NOT NULL,
  mtime_ms INTEGER,
  synced_at_ms INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS mir_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
"#;

/// SQLite query/projection store that applies sync signals.
pub struct SqliteStore {
    store_id: String,
    conn: Connection,
}

impl SqliteStore {
    pub fn open(store_id: impl Into<String>, db_path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let store_id = store_id.into();
        let conn = Connection::open(db_path.as_ref()).map_err(|source| StoreError::Sqlite {
            store_id: store_id.clone(),
            source,
        })?;
        conn.execute_batch(SCHEMA)
            .map_err(|source| StoreError::Sqlite {
                store_id: store_id.clone(),
                source,
            })?;
        Ok(Self { store_id, conn })
    }

    pub fn open_in_memory(store_id: impl Into<String>) -> Result<Self, StoreError> {
        let store_id = store_id.into();
        let conn = Connection::open_in_memory().map_err(|source| StoreError::Sqlite {
            store_id: store_id.clone(),
            source,
        })?;
        conn.execute_batch(SCHEMA)
            .map_err(|source| StoreError::Sqlite {
                store_id: store_id.clone(),
                source,
            })?;
        Ok(Self { store_id, conn })
    }

    fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    fn clear_all(&self) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM records", [])
            .map_err(|source| StoreError::Sqlite {
                store_id: self.store_id.clone(),
                source,
            })?;
        Ok(())
    }

    fn upsert(&self, record: &StoreRecord) -> Result<(), StoreError> {
        let synced_at = Self::now_ms();
        self.conn
            .execute(
                "INSERT INTO records (id, kind, content, mtime_ms, synced_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                   kind = excluded.kind,
                   content = excluded.content,
                   mtime_ms = excluded.mtime_ms,
                   synced_at_ms = excluded.synced_at_ms",
                params![
                    record.id,
                    record.kind,
                    record.content,
                    record.mtime_ms,
                    synced_at
                ],
            )
            .map_err(|source| StoreError::Sqlite {
                store_id: self.store_id.clone(),
                source,
            })?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM records WHERE id = ?1", params![id])
            .map_err(|source| StoreError::Sqlite {
                store_id: self.store_id.clone(),
                source,
            })?;
        Ok(())
    }
}

impl Store for SqliteStore {
    fn store_id(&self) -> &str {
        &self.store_id
    }

    fn capabilities(&self) -> StoreCapabilities {
        StoreCapabilities {
            observe_source: false,
            observe_sink: true,
            source_read: false,
            query_surface: true,
        }
    }

    fn apply(&mut self, signal: &SyncSignal, records: &[StoreRecord]) -> Result<(), StoreError> {
        match &signal.scope {
            SyncScope::Full | SyncScope::Content => {
                self.clear_all()?;
                for record in records {
                    self.upsert(record)?;
                }
                Ok(())
            }
            SyncScope::Incremental(changes) => {
                let by_id: std::collections::BTreeMap<&str, &StoreRecord> =
                    records.iter().map(|r| (r.id.as_str(), r)).collect();
                for change in changes {
                    match change.kind {
                        ChangeKind::Deleted => self.delete(&change.id)?,
                        ChangeKind::Created | ChangeKind::Modified => {
                            if let Some(record) = by_id.get(change.id.as_str()) {
                                self.upsert(record)?;
                            } else if let Some(preview) = &signal.preview {
                                if preview.id == change.id {
                                    self.upsert(&StoreRecord {
                                        id: preview.id.clone(),
                                        kind: preview.kind.clone(),
                                        content: preview.content.clone(),
                                        mtime_ms: None,
                                    })?;
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn pull_record(&self, id: &str) -> Result<Option<StoreRecord>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, content, mtime_ms FROM records WHERE id = ?1",
            )
            .map_err(|source| StoreError::Sqlite {
                store_id: self.store_id.clone(),
                source,
            })?;
        let mut rows = stmt
            .query(params![id])
            .map_err(|source| StoreError::Sqlite {
                store_id: self.store_id.clone(),
                source,
            })?;
        match rows.next().map_err(|source| StoreError::Sqlite {
            store_id: self.store_id.clone(),
            source,
        })? {
            Some(row) => Ok(Some(StoreRecord {
                id: row.get(0).map_err(|source| StoreError::Sqlite {
                    store_id: self.store_id.clone(),
                    source,
                })?,
                kind: row.get(1).map_err(|source| StoreError::Sqlite {
                    store_id: self.store_id.clone(),
                    source,
                })?,
                content: row.get(2).map_err(|source| StoreError::Sqlite {
                    store_id: self.store_id.clone(),
                    source,
                })?,
                mtime_ms: row.get(3).map_err(|source| StoreError::Sqlite {
                    store_id: self.store_id.clone(),
                    source,
                })?,
            })),
            None => Ok(None),
        }
    }
}
