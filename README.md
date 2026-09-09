# Imp (Rust implementation)

Rust implementation of the Imp DAG transmission format. Language-neutral specs live in **[imp-spec](https://github.com/silentorb/imp-spec)**.

## Crates (foundation)

| Rust crate | imp-ts package |
| --- | --- |
| `imp_core_types` | `imp-core-types` |
| `imp_registry` | `imp-registry` |
| `imp_collection_transforms` | `imp-collection-transforms` |
| `imp_pathing` | `imp-pathing` |
| `imp_execution` | `imp-execution` |

## Development

From repo root:

```bash
cargo test
cargo clippy -- -D warnings
```

See [AGENTS.md](./AGENTS.md) and [docs/rust-style.md](./docs/rust-style.md).
