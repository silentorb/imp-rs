//! Session: text → SQLite wire-up, cold sync, steady watch, pull (Miro profile).

use crate::signal::SyncSignal;
use crate::sqlite::SqliteStore;
use crate::store::{Store, StoreError, StoreRecord, StoreRegistry};
use crate::text_file::{unknown_kind_fn, KindFn, TextFileStore};
use crate::wire::{default_sync_graph, wire_sync_graph, ObserveEdge, WiredSync, WireError};
use imp_core_types::Graph;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const DEFAULT_TEXT_STORE_ID: &str = "authoring";
pub const DEFAULT_SQLITE_STORE_ID: &str = "projection";

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("{0}")]
    Store(#[from] StoreError),
    #[error("{0}")]
    Wire(#[from] WireError),
    #[error("{0}")]
    Message(String),
}

/// Options for opening a text → SQLite session (Miro profile).
pub struct OpenOptions {
    pub content_root: PathBuf,
    pub sqlite_path: PathBuf,
    pub kind_fn: KindFn,
    /// When `None`, use [`default_sync_graph`].
    pub sync_graph: Option<Graph>,
    pub start_watch: bool,
}

impl OpenOptions {
    pub fn new(content_root: impl Into<PathBuf>, sqlite_path: impl Into<PathBuf>) -> Self {
        Self {
            content_root: content_root.into(),
            sqlite_path: sqlite_path.into(),
            kind_fn: unknown_kind_fn(),
            sync_graph: None,
            start_watch: true,
        }
    }

    pub fn with_kind_fn(mut self, kind_fn: KindFn) -> Self {
        self.kind_fn = kind_fn;
        self
    }

    pub fn with_sync_graph(mut self, graph: Graph) -> Self {
        self.sync_graph = Some(graph);
        self
    }

    pub fn with_start_watch(mut self, start_watch: bool) -> Self {
        self.start_watch = start_watch;
        self
    }
}

/// Running session: authoring text store observing into SQLite projection.
pub struct Session {
    text: TextFileStore,
    sqlite: SqliteStore,
    wired: WiredSync,
    query_store_id: String,
}

impl Session {
    /// Open stores, wire sync graph, optionally start FS watch, run cold `full` sync.
    pub fn open(options: OpenOptions) -> Result<Self, SessionError> {
        let mut text = TextFileStore::open(
            DEFAULT_TEXT_STORE_ID,
            options.content_root,
            options.kind_fn,
        )?;
        let sqlite = SqliteStore::open(DEFAULT_SQLITE_STORE_ID, &options.sqlite_path)?;

        let mut registry = StoreRegistry::new();
        // Temporary boxes only for wire-up capability validation — we move stores into Self after.
        // Validate against standalone capability snapshots instead.
        let graph = options
            .sync_graph
            .unwrap_or_else(|| default_sync_graph(DEFAULT_TEXT_STORE_ID, DEFAULT_SQLITE_STORE_ID));

        // Build a validation registry with the same store ids/capabilities.
        registry.insert(Box::new(CapabilityProbe {
            id: DEFAULT_TEXT_STORE_ID.to_string(),
            caps: text.capabilities(),
        }));
        registry.insert(Box::new(CapabilityProbe {
            id: DEFAULT_SQLITE_STORE_ID.to_string(),
            caps: sqlite.capabilities(),
        }));
        registry.set_query_store_id(DEFAULT_SQLITE_STORE_ID);
        let wired = wire_sync_graph(&graph, &registry)?;

        if options.start_watch {
            text.start_watch()?;
        }

        let mut session = Self {
            text,
            sqlite,
            wired,
            query_store_id: DEFAULT_SQLITE_STORE_ID.to_string(),
        };
        session.cold_sync()?;
        Ok(session)
    }

    /// Open an in-memory SQLite projection (tests / ephemeral tools).
    pub fn open_in_memory(
        content_root: impl Into<PathBuf>,
        kind_fn: KindFn,
        start_watch: bool,
    ) -> Result<Self, SessionError> {
        let mut text = TextFileStore::open(DEFAULT_TEXT_STORE_ID, content_root, kind_fn)?;
        let sqlite = SqliteStore::open_in_memory(DEFAULT_SQLITE_STORE_ID)?;
        let mut registry = StoreRegistry::new();
        registry.insert(Box::new(CapabilityProbe {
            id: DEFAULT_TEXT_STORE_ID.to_string(),
            caps: text.capabilities(),
        }));
        registry.insert(Box::new(CapabilityProbe {
            id: DEFAULT_SQLITE_STORE_ID.to_string(),
            caps: sqlite.capabilities(),
        }));
        let graph = default_sync_graph(DEFAULT_TEXT_STORE_ID, DEFAULT_SQLITE_STORE_ID);
        let wired = wire_sync_graph(&graph, &registry)?;
        if start_watch {
            text.start_watch()?;
        }
        let mut session = Self {
            text,
            sqlite,
            wired,
            query_store_id: DEFAULT_SQLITE_STORE_ID.to_string(),
        };
        session.cold_sync()?;
        Ok(session)
    }

    pub fn content_root(&self) -> &Path {
        self.text.content_root()
    }

    pub fn wired(&self) -> &WiredSync {
        &self.wired
    }

    pub fn query_store_id(&self) -> &str {
        &self.query_store_id
    }

    /// Cold full sync from text source(s) into sinks along wired edges.
    pub fn cold_sync(&mut self) -> Result<(), SessionError> {
        // Collect unique source store ids that observe into sinks.
        let sources: Vec<String> = self
            .wired
            .edges
            .iter()
            .map(|e| e.source_store_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        for source_id in sources {
            let signal = SyncSignal::full(&source_id);
            self.dispatch(&signal)?;
        }
        Ok(())
    }

    /// Drain FS watch events and apply incremental signals.
    pub fn poll_watch(&mut self) -> Result<usize, SessionError> {
        let Some(signal) = self.text.drain_watch_signal()? else {
            return Ok(0);
        };
        self.dispatch(&signal)?;
        Ok(1)
    }

    /// Pull an opaque record from the designated query store (SQLite).
    pub fn pull_record(&self, id: &str) -> Result<Option<StoreRecord>, SessionError> {
        Ok(self.sqlite.pull_record(id)?)
    }

    fn dispatch(&mut self, signal: &SyncSignal) -> Result<(), SessionError> {
        let edges: Vec<ObserveEdge> = self
            .wired
            .edges
            .iter()
            .filter(|e| e.source_store_id == signal.source)
            .cloned()
            .collect();
        for edge in edges {
            let records = if edge.source_store_id == self.text.store_id() {
                self.text.read_for_signal(signal)?
            } else {
                return Err(SessionError::Message(format!(
                    "unsupported source store `{}`",
                    edge.source_store_id
                )));
            };
            if edge.sink_store_id == self.sqlite.store_id() {
                self.sqlite.apply(signal, &records)?;
            } else {
                return Err(SessionError::Message(format!(
                    "unsupported sink store `{}`",
                    edge.sink_store_id
                )));
            }
        }
        Ok(())
    }
}

/// Minimal store used only for wire-up capability checks.
struct CapabilityProbe {
    id: String,
    caps: crate::store::StoreCapabilities,
}

impl Store for CapabilityProbe {
    fn store_id(&self) -> &str {
        &self.id
    }

    fn capabilities(&self) -> crate::store::StoreCapabilities {
        self.caps
    }
}
