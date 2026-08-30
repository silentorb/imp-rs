# Imp (Rust binding)

Rust implementation of the Imp DAG transmission format. Language-neutral specs live in sibling **[imp-spec](../imp-spec/)**.

## Crates (foundation)

| Rust crate | imp-ts package |
| --- | --- |
| `imp_core_types` | `imp-core-types` |
| `imp_registry` | `imp-registry` |
| `imp_collection_transforms` | `imp-collection-transforms` |
| `imp_pathing` | `imp-pathing` |

`imp_execution` is planned for a follow-up session.

## Development

From repo root:

```bash
cargo test
cargo clippy -- -D warnings
```

In **silentorb-workbench**: `bash scripts/run-in-imp-rust.sh test` (requires Rust toolchain or the imp-rust devcontainer).

See [AGENTS.md](./AGENTS.md) and [docs/rust-style.md](./docs/rust-style.md).
