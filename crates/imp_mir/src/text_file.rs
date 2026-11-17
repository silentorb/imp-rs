//! Text-file observe source for Miro-profile authoring trees.

use crate::signal::{ChangeKind, IncrementalChange, RecordId, SyncScope, SyncSignal};
use crate::store::{Store, StoreCapabilities, StoreError, StoreRecord};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::SystemTime;

/// Maps relative path + file content to an opaque record kind string.
pub type KindFn = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;

/// File-tree store: content root as authoring SoT; can emit observe signals and source-read.
pub struct TextFileStore {
    store_id: String,
    content_root: PathBuf,
    kind_fn: KindFn,
    watcher: Option<RecommendedWatcher>,
    events_rx: Option<Receiver<notify::Result<Event>>>,
}

impl TextFileStore {
    pub fn open(
        store_id: impl Into<String>,
        content_root: impl Into<PathBuf>,
        kind_fn: KindFn,
    ) -> Result<Self, StoreError> {
        let store_id = store_id.into();
        let content_root = content_root.into();
        if !content_root.is_dir() {
            return Err(StoreError::Message {
                store_id: store_id.clone(),
                message: format!("content root is not a directory: {}", content_root.display()),
            });
        }
        Ok(Self {
            store_id,
            content_root,
            kind_fn,
            watcher: None,
            events_rx: None,
        })
    }

    pub fn content_root(&self) -> &Path {
        &self.content_root
    }

    /// Start recursive file-tree watching (observeOut).
    pub fn start_watch(&mut self) -> Result<(), StoreError> {
        let (tx, rx): (Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) =
            mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(|err| StoreError::Message {
            store_id: self.store_id.clone(),
            message: format!("failed to create watcher: {err}"),
        })?;
        watcher
            .watch(&self.content_root, RecursiveMode::Recursive)
            .map_err(|err| StoreError::Message {
                store_id: self.store_id.clone(),
                message: format!("failed to watch {}: {err}", self.content_root.display()),
            })?;
        self.watcher = Some(watcher);
        self.events_rx = Some(rx);
        Ok(())
    }

    /// Drain pending FS events into an incremental sync signal, if any.
    pub fn drain_watch_signal(&self) -> Result<Option<SyncSignal>, StoreError> {
        let Some(rx) = &self.events_rx else {
            return Ok(None);
        };
        let mut changes: BTreeMap<RecordId, ChangeKind> = BTreeMap::new();
        while let Ok(event) = rx.try_recv() {
            let event = event.map_err(|err| StoreError::Message {
                store_id: self.store_id.clone(),
                message: format!("watch error: {err}"),
            })?;
            let change_kind = match event.kind {
                EventKind::Create(_) => ChangeKind::Created,
                EventKind::Modify(_) => ChangeKind::Modified,
                EventKind::Remove(_) => ChangeKind::Deleted,
                _ => continue,
            };
            for path in event.paths {
                if path.is_dir() {
                    continue;
                }
                let Some(rel) = relativize(&self.content_root, &path) else {
                    continue;
                };
                if should_skip(&rel) {
                    continue;
                }
                changes
                    .entry(rel)
                    .and_modify(|existing| {
                        // Prefer Deleted if mixed; else latest non-create wins as Modified.
                        *existing = match (*existing, change_kind) {
                            (_, ChangeKind::Deleted) => ChangeKind::Deleted,
                            (ChangeKind::Deleted, _) => ChangeKind::Deleted,
                            (ChangeKind::Created, ChangeKind::Created) => ChangeKind::Created,
                            _ => ChangeKind::Modified,
                        };
                    })
                    .or_insert(change_kind);
            }
        }
        if changes.is_empty() {
            return Ok(None);
        }
        let changes = changes
            .into_iter()
            .map(|(id, kind)| IncrementalChange { id, kind })
            .collect();
        Ok(Some(SyncSignal::incremental(&self.store_id, changes)))
    }

    fn list_records(&self) -> Result<Vec<StoreRecord>, StoreError> {
        let mut records = Vec::new();
        collect_files(&self.content_root, &self.content_root, &mut records, &self.kind_fn)
            .map_err(|source| StoreError::Io {
                store_id: self.store_id.clone(),
                source,
            })?;
        Ok(records)
    }

    fn read_one(&self, rel: &str) -> Result<Option<StoreRecord>, StoreError> {
        let path = self.content_root.join(rel);
        if !path.is_file() {
            return Ok(None);
        }
        read_record(&self.content_root, &path, &self.kind_fn).map(Some).map_err(|source| {
            StoreError::Io {
                store_id: self.store_id.clone(),
                source,
            }
        })
    }
}

impl Store for TextFileStore {
    fn store_id(&self) -> &str {
        &self.store_id
    }

    fn capabilities(&self) -> StoreCapabilities {
        StoreCapabilities {
            observe_source: true,
            observe_sink: false,
            source_read: true,
            query_surface: false,
        }
    }

    fn read_for_signal(&self, signal: &SyncSignal) -> Result<Vec<StoreRecord>, StoreError> {
        match &signal.scope {
            SyncScope::Full | SyncScope::Content => self.list_records(),
            SyncScope::Incremental(changes) => {
                let mut out = Vec::new();
                for change in changes {
                    if change.kind == ChangeKind::Deleted {
                        continue;
                    }
                    if let Some(record) = self.read_one(&change.id)? {
                        out.push(record);
                    }
                }
                Ok(out)
            }
        }
    }
}

fn default_kind(_path: &str, _content: &str) -> String {
    "unknown".to_string()
}

/// Kind function that always returns `"unknown"`.
pub fn unknown_kind_fn() -> KindFn {
    Arc::new(default_kind)
}

fn should_skip(rel: &str) -> bool {
    rel.split(['/', '\\']).any(|part| part.starts_with('.'))
}

fn relativize(root: &Path, path: &Path) -> Option<RecordId> {
    path.strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
}

fn mtime_ms(meta: &fs::Metadata) -> Option<i64> {
    meta.modified().ok().and_then(|t| {
        t.duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_millis() as i64)
    })
}

fn read_record(root: &Path, path: &Path, kind_fn: &KindFn) -> Result<StoreRecord, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let rel = relativize(root, path).unwrap_or_else(|| path.display().to_string());
    let meta = fs::metadata(path)?;
    let kind = kind_fn(&rel, &content);
    Ok(StoreRecord {
        id: rel,
        kind,
        content,
        mtime_ms: mtime_ms(&meta),
    })
}

fn collect_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<StoreRecord>,
    kind_fn: &KindFn,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, out, kind_fn)?;
        } else if path.is_file() {
            out.push(read_record(root, &path, kind_fn)?);
        }
    }
    Ok(())
}
