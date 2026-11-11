//! Mir sync fabric and Miro text→SQLite→pull for Imp hosts.
//!
//! Specs: [Mir](https://github.com/silentorb/mir). This crate does not make Imp core
//! packages depend on Mir; it is a host binding that uses Imp graphs for sync wiring.

mod session;
mod signal;
mod sqlite;
mod store;
mod text_file;
mod wire;

pub use session::{
    MiroError, MiroOpenOptions, MiroSession, DEFAULT_SQLITE_STORE_ID, DEFAULT_TEXT_STORE_ID,
};
pub use signal::{
    ChangeKind, IncrementalChange, Preview, RecordId, SyncScope, SyncSignal,
};
pub use sqlite::SqliteStore;
pub use store::{Store, StoreCapabilities, StoreError, StoreRecord, StoreRegistry};
pub use text_file::{unknown_kind_fn, KindFn, TextFileStore};
pub use wire::{
    default_miro_graph, wire_sync_graph, ObserveEdge, WireError, WiredSync, MIR_SYNC_STORE,
    PORT_OBSERVE_IN, PORT_OBSERVE_OUT, PORT_STORE_ID,
};
