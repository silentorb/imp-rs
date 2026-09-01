# AGENTS Guide — Imp (Rust binding)

## Repository purpose

**Imp** is a universal transmission format for directed acyclic graphs (DAGs). This repo is the **Rust binding**. Language-neutral specs live in sibling **[imp-spec](../imp-spec/)** — read those for data model and behavior; this repo implements them.

| Rust crate | imp-ts package | Role |
| --- | --- | --- |
| `crates/imp_core_types/` | `imp-core-types` | Core graph + library types; core boundary `NodeLibrary` |
| `crates/imp_registry/` | `imp-registry` | Load `NodeLibrary` values and look up `NodeType`s |
| `crates/imp_typecheck/` | `imp-typecheck` | Static signal-type checking |
| `crates/imp_graph_resolve/` | `imp-graph-resolve` | Late nested graph resolution |
| `crates/imp_collection_transforms/` | `imp-collection-transforms` | Collection combinator `NodeLibrary` |
| `crates/imp_pathing/` | `imp-pathing` | GQL-like path operator `NodeLibrary` |
| `crates/imp_execution/` | `imp-execution` | Runtime consumer (deferred) |

**Imp Translator** specs (SQL, React Flow) and runtime docs live in [`imp-ts/docs/features/`](../imp-ts/docs/features/), not imp-spec.

## Specs vs code

| Layer | Location |
| --- | --- |
| Language-neutral specs | [`../imp-spec/docs/packages/`](../imp-spec/docs/packages/) |
| Rust implementation | `crates/*/src` |

When specs and code disagree, update imp-spec or the code explicitly.

## Project context

- Run from repo root: `cargo test`, `cargo clippy -- -D warnings`.
- **Coverage:** [docs/coverage.md](./docs/coverage.md) — heuristic line targets in `coverage-targets.toml`. From silentorb-workbench: `bash scripts/coverage-imp-rust.sh --changed` after editing `src/` or `tests/`.
- Tests live in `crates/<crate>/tests/` (not inline in `src/`). See [docs/rust-style.md](./docs/rust-style.md).
- Code style: [docs/rust-style.md](./docs/rust-style.md).
- TypeScript reference binding: [`../imp-ts/`](../imp-ts/).

## Feature documentation (imp-spec)

| If your task involves… | Read |
| --- | --- |
| Core graph model | [graph-model](../imp-spec/docs/packages/imp-core-types/graph-model.md) |
| Node type libraries | [node-libraries](../imp-spec/docs/packages/imp-core-types/node-libraries.md) |
| Graph libraries | [graph-libraries](../imp-spec/docs/packages/imp-core-types/graph-libraries.md) |
| Graph resolution | [resolve](../imp-spec/docs/packages/imp-graph-resolve/resolve.md) |
| Static type checking | [type-system](../imp-spec/docs/packages/imp-typecheck/type-system.md) |
| Registry | [registry](../imp-spec/docs/packages/imp-registry/registry.md) |
| Collection transforms | [collection-transforms](../imp-spec/docs/packages/imp-collection-transforms/collection-transforms.md) |
| Pathing | [pathing](../imp-spec/docs/packages/imp-pathing/pathing.md) |

Cross-package overview: [`../imp-spec/docs/overview/`](../imp-spec/docs/overview/).

## Working conventions

- Focused changes only; avoid unrelated refactors.
- **Prototypal stage — no backwards compatibility.** Delete old shapes; migrate consumers in the same change.
- Crate names use **snake_case** (`imp_core_types`); imp-ts npm packages use hyphenated `imp-*`.
- Follow [docs/rust-style.md](./docs/rust-style.md): plain structs, free functions, immutable inputs / owned outputs.

## Versioning

Cargo workspace version (`0.2.0`) aligns with imp-ts npm epoch. Version bumps for npm packages are orchestrated from **silentorb-workbench** (`bash scripts/bump-version.sh`); this repo has no npm packages.

## Workbench integration

In **silentorb-workbench**, this repo mounts at `.mnt/imp-rust/` (optional; host default `../imp-rust`, or `IMP_RUST_REPO`). Specs: `.mnt/imp-spec/`. Run cargo via `bash scripts/run-in-imp-rust.sh …` from the workbench root (ephemeral imp-rust Compose container when the workbench shell has no local `cargo`). Coverage: `bash scripts/coverage-imp-rust.sh`.

Use the repo [`.devcontainer/`](./.devcontainer/) for a standalone Rust toolchain, or install `rustup` in the main workbench shell.
