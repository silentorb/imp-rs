# AGENTS Guide — Imp (Rust implementation)

## Repository purpose

**Imp** is a universal transmission format for directed acyclic graphs (DAGs). This repo is the **Rust implementation**. Language-neutral specs live in **[imp-spec](https://github.com/silentorb/imp-spec)** — read those for data model and behavior; this repo implements them.

| Rust crate | imp-ts package | Role |
| --- | --- | --- |
| `crates/imp_core_types/` | `imp-core-types` | Core graph + library types; core boundary `NodeLibrary` |
| `crates/imp_registry/` | `imp-registry` | Load `NodeLibrary` values and look up `NodeType`s |
| `crates/imp_typecheck/` | `imp-typecheck` | Static type checking |
| `crates/imp_graph_resolve/` | `imp-graph-resolve` | Late nested graph resolution |
| `crates/imp_collection_transforms/` | `imp-collection-transforms` | Collection combinator `NodeLibrary` |
| `crates/imp_pathing/` | `imp-pathing` | GQL-like path operator `NodeLibrary` |
| `crates/imp_execution/` | `imp-execution` | Runtime consumer (deferred) |

**Imp Translator** specs (SQL, React Flow) and runtime docs live in [imp-ts `docs/features/`](https://github.com/silentorb/imp-ts/tree/main/docs/features/), not imp-spec.

## Specs vs code

| Layer | Location |
| --- | --- |
| Language-neutral specs | [imp-spec `docs/packages/`](https://github.com/silentorb/imp-spec/tree/main/docs/packages/) |
| Rust implementation | `crates/*/src` |

When specs and code disagree, update imp-spec or the code explicitly.

## Project context

- Run from repo root: `cargo test`, `cargo clippy -- -D warnings`.
- **Coverage:** [docs/coverage.md](./docs/coverage.md) — heuristic line targets in `coverage-targets.toml`. After editing `src/` or `tests/`, run coverage via `cargo llvm-cov` as documented there.
- Tests live in `crates/<crate>/tests/` (not inline in `src/`). See [docs/rust-style.md](./docs/rust-style.md).
- Code style: [docs/rust-style.md](./docs/rust-style.md).
- TypeScript reference implementation: [imp-ts](https://github.com/silentorb/imp-ts).

## Feature documentation (imp-spec)

| If your task involves… | Read |
| --- | --- |
| Core graph model | [graph-model](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-core-types/graph-model.md) |
| Node type libraries | [node-libraries](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-core-types/node-libraries.md) |
| Graph libraries | [graph-libraries](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-core-types/graph-libraries.md) |
| Graph resolution | [resolve](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-graph-resolve/resolve.md) |
| Static type checking | [type-system](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-typecheck/type-system.md) |
| Registry | [registry](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-registry/registry.md) |
| Collection transforms | [collection-transforms](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-collection-transforms/collection-transforms.md) |
| Pathing | [pathing](https://github.com/silentorb/imp-spec/blob/main/docs/packages/imp-pathing/pathing.md) |
| Mir / Miro sync (`imp_mir`) | [Mir repo](https://github.com/silentorb/mir) (`docs/features/`); the `imp_mir` crate lives in [tome-rust-server](https://github.com/silentorb/tome-rust-server) (`crates/imp_mir`) |

Cross-package overview: [imp-spec `docs/overview/`](https://github.com/silentorb/imp-spec/tree/main/docs/overview/). Imp **specs** must not mention Mir; `imp_mir` is a separate host binding maintained in tome-rust-server, not in this workspace.

## Working conventions

- Focused changes only; avoid unrelated refactors.
- **Prototypal stage — no backwards compatibility.** Delete old shapes; migrate consumers in the same change.
- Crate names use **snake_case** (`imp_core_types`); imp-ts npm packages use hyphenated `imp-*`.
- Follow [docs/rust-style.md](./docs/rust-style.md): plain structs, free functions, immutable inputs / owned outputs, and [Errors](./docs/rust-style.md#errors).

## Versioning

Cargo workspace version (`0.2.0`) aligns with the imp-ts npm epoch. This repo has no npm packages.
