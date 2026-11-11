//! Store registry and capability-bearing adapters.

use crate::signal::{RecordId, SyncSignal};
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

/// Capability flags a store may provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StoreCapabilities {
    pub observe_source: bool,
    pub observe_sink: bool,
    pub source_read: bool,
    pub query_surface: bool,
}

/// A durable record read from a source or query store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecord {
    pub id: RecordId,
    pub kind: String,
    pub content: String,
    pub mtime_ms: Option<i64>,
}

/// Runtime store adapter.
pub trait Store: Send {
    fn store_id(&self) -> &str;
    fn capabilities(&self) -> StoreCapabilities;

    /// Expand a signal into concrete records (source-read).
    fn read_for_signal(&self, signal: &SyncSignal) -> Result<Vec<StoreRecord>, StoreError> {
        let _ = signal;
        Err(StoreError::Unsupported {
            store_id: self.store_id().to_string(),
            operation: "source_read",
        })
    }

    /// Apply a sync signal (observe sink).
    fn apply(&mut self, signal: &SyncSignal, records: &[StoreRecord]) -> Result<(), StoreError> {
        let _ = (signal, records);
        Err(StoreError::Unsupported {
            store_id: self.store_id().to_string(),
            operation: "observe_sink",
        })
    }

    /// Pull one record from a query surface by opaque id.
    fn pull_record(&self, id: &str) -> Result<Option<StoreRecord>, StoreError> {
        let _ = id;
        Err(StoreError::Unsupported {
            store_id: self.store_id().to_string(),
            operation: "query_surface",
        })
    }
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("store `{store_id}` does not support `{operation}`")]
    Unsupported { store_id: String, operation: &'static str },
    #[error("store `{store_id}`: {message}")]
    Message { store_id: String, message: String },
    #[error("io error in store `{store_id}`: {source}")]
    Io {
        store_id: String,
        #[source]
        source: std::io::Error,
    },
    #[error("sqlite error in store `{store_id}`: {source}")]
    Sqlite {
        store_id: String,
        #[source]
        source: rusqlite::Error,
    },
}

/// Opened store adapters keyed by registry id.
#[derive(Default)]
pub struct StoreRegistry {
    stores: BTreeMap<String, Box<dyn Store>>,
    query_store_id: Option<String>,
}

impl StoreRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, store: Box<dyn Store>) {
        self.stores.insert(store.store_id().to_string(), store);
    }

    pub fn set_query_store_id(&mut self, store_id: impl Into<String>) {
        self.query_store_id = Some(store_id.into());
    }

    pub fn query_store_id(&self) -> Option<&str> {
        self.query_store_id.as_deref()
    }

    pub fn get(&self, store_id: &str) -> Option<&dyn Store> {
        self.stores.get(store_id).map(|s| s.as_ref())
    }

    pub fn get_mut(&mut self, store_id: &str) -> Option<&mut dyn Store> {
        match self.stores.get_mut(store_id) {
            Some(s) => Some(s.as_mut()),
            None => None,
        }
    }

    pub fn contains(&self, store_id: &str) -> bool {
        self.stores.contains_key(store_id)
    }

    pub fn store_ids(&self) -> impl Iterator<Item = &str> {
        self.stores.keys().map(|s| s.as_str())
    }
}

impl fmt::Debug for StoreRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreRegistry")
            .field("stores", &self.stores.keys().collect::<Vec<_>>())
            .field("query_store_id", &self.query_store_id)
            .finish()
    }
}
