# Rust style guide

Living style for **imp-rs** crates. Adapted from [silentorb/margen `docs/rust-style.md`](https://github.com/silentorb/margen/blob/main/docs/rust-style.md) (general guidelines only).

## Orientation

Prefer **lowest-common-denominator FP**: plain data, free functions, and many **simple flat steps** instead of few deeply nested mutating steps. That keeps ownership and lifetimes obvious.

## Data vs functions

- Prefer plain `struct`s with public fields plus **free functions**.
- Keep inherent methods minimal (constructors, trivial accessors).
- Do not grow domain types with methods in every dependent crate—dependents add free functions.

## Ownership and mutation

- Prefer **immutable inputs / owned outputs**.
- Mutate only in tight local steps; avoid long-lived shared `&mut` bags across nested helpers.
- When the borrow checker blocks simultaneous mutable borrows across nested helpers, **stage flat passes** (collect, then apply)—do not reach for interior mutability by default.

## Collections

- Prefer **homogeneous** collections and normalized records (SQL-like).
- Default to `Vec`, `HashMap` / `BTreeMap`, `HashSet` / `BTreeSet`.
- Use `BTreeMap` / `BTreeSet` when iteration order must be deterministic across platforms.
- Prefer `Vec` unless fixed-size semantics are load-bearing.

## Errors

- Prefer `Result` / explicit status for **expected** failure (graph errors, missing ports, duplicate type ids).
- Panic only for programmer invariants or documented fail-fast aborts.
- Prefer precise, explicit error flows over broad catch-all handling.
- Propagate errors with `?` unless the current layer can meaningfully recover.
- Handle errors locally only with an intentional recovery or translation strategy.
- Preserve error causes and useful context.
- Use typed errors when callers need to distinguish failure cases.
- Avoid silently ignored errors, arbitrary fallbacks, and unjustified `unwrap()`/`expect()`.

## Crate layout

- **imp-ts package ≡ Cargo crate** (one crate per npm package; no umbrella/facade crate).
- Internal TypeScript modules become Rust modules inside that crate.
- Extra focused crates are welcome; do not collapse packages.
- **`imp_execution`** — demand-driven graph interpreter; see crate module docs for performance tier notes.

## Friction rule

If an imp-ts pattern forces awkward Rust (shared mutable eval context, polymorphic host bags that will not stage cleanly), **stop and report** instead of inventing a forced workaround.

## Naming and layout

- Crates live under `crates/` with **snake_case** names (`imp_core_types`, `imp_registry`, …).
- Same for `[package] name`, workspace members, and dependency keys.
- imp-ts npm packages use hyphenated `imp-*`; map via [AGENTS.md](../AGENTS.md).
- Unix (LF) line endings; follow `.editorconfig`.

## Tests

- Keep **tests separate from implementation** — no `#[cfg(test)] mod tests` in `src/`.
- Per-crate integration tests live under `crates/<crate>/tests/*.rs` (Cargo’s standard layout).
- Tests use the crate **public API** only (same surface a downstream consumer would use).
- Name test files after the module or capability under test (`core_library.rs`, `registry.rs`, `library.rs`, …).
- Cross-crate scenarios belong in the test file of the crate being verified; add `[dev-dependencies]` for helpers (e.g. `imp_registry` in library crate tests).
- Workspace-wide smoke tests may be added under a dedicated test crate later; prefer per-crate `tests/` until then.

## imp-rs specifics

- Spec-first: [imp-spec `docs/packages/`](https://github.com/silentorb/imp-spec/tree/main/docs/packages/) is authoritative; imp-ts is reference when the spec is silent.
- Align public function names with imp-ts where sensible (`load_library`, `core_node_library`, …).
- Execution API (`execute_graph`, `ExecutionHost`) lives in `imp_execution`.
