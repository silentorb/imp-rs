# imp_mir

Rust host binding for **[Mir](https://github.com/silentorb/mir)** (Imp-configured multi-store sync) and the **Miro** profile (text files → database projection → pull app memory).

Language-neutral requirements live in the Mir repo (`docs/features/imp-sync-wiring.md`, `sync-nodes.md`, `miro.md`). This crate implements them; it does **not** teach Imp core crates or [imp-spec](https://github.com/silentorb/imp-spec) about Mir.

## MVP surface

- Store registry + capability checks
- `SyncSignal` with full / incremental / content scopes
- Boot-only wire-up for `mir.sync.store` (file-tree observe)
- Generic text-file store + SQLite projection store
- `Session`: default text → SQLite, cold `full` sync, steady file watch, opaque `pull_record`

Out of scope here: `mir.sync.gitWatch`, multi-source fan-in, host domain codecs (e.g. Aura JSON → Imp `Graph`).

## Specs

| Spec | Role |
|------|------|
| [Mir](https://github.com/silentorb/mir) | Sync fabric + Miro profile |
| Imp graph model | Encoding for observe topology (`storeId`-only node inputs) |
