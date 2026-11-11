//! Sync signals — Mir change units between stores.

use std::fmt;

/// Opaque record identity at the Mir layer (hosts choose encoding).
pub type RecordId = String;

/// How a record changed in an incremental scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
}

/// What changed — conceptual Mir scopes; hosts map concrete encodings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncScope {
    /// Rebuild / reapply from the source(s) as a whole.
    Full,
    /// Opaque record identities and change kinds.
    Incremental(Vec<IncrementalChange>),
    /// Non-record configuration or content-model material changed.
    Content,
}

/// One opaque record change inside an incremental scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalChange {
    pub id: RecordId,
    pub kind: ChangeKind,
}

/// Optional opaque preview so sinks can apply without waiting on durable source materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub id: RecordId,
    pub kind: String,
    pub content: String,
}

/// Sync signal emitted by an observe source toward observe sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncSignal {
    /// Registry id of the originating store.
    pub source: String,
    pub scope: SyncScope,
    pub preview: Option<Preview>,
}

impl SyncSignal {
    pub fn full(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            scope: SyncScope::Full,
            preview: None,
        }
    }

    pub fn incremental(source: impl Into<String>, changes: Vec<IncrementalChange>) -> Self {
        Self {
            source: source.into(),
            scope: SyncScope::Incremental(changes),
            preview: None,
        }
    }
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Modified => write!(f, "modified"),
            Self::Deleted => write!(f, "deleted"),
        }
    }
}
