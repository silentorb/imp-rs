# Test coverage (imp-rust)

Heuristic line-coverage targets for agents and occasional maintenance. Coverage is **advisory** — not a CI gate.

## Targets

Configured in [`coverage-targets.toml`](../coverage-targets.toml) at repo root:

| Scope | Field | Default |
| ----- | ----- | ------- |
| Workspace | `line_target` | 85% — agent heuristic goal |
| Workspace | `line_minimum` | 70% — report as needs attention |
| Per crate | `[crates.<name>].line_target` | falls back to workspace target |

## Running coverage

From **silentorb-workbench** (ephemeral imp-rust container via Compose):

```bash
bash scripts/coverage-imp-rust.sh           # full workspace report
bash scripts/coverage-imp-rust.sh --changed # highlight git-changed src files
bash scripts/coverage-imp-rust.sh --html    # also write target/llvm-cov/html/
bash scripts/coverage-imp-rust.sh --strict  # exit non-zero below line_minimum
```

From this repo directly (standalone devcontainer or local `cargo`):

```bash
cargo llvm-cov --workspace --json --summary-only --output-path coverage.json
cargo llvm-cov report --summary-only
```

## Agent workflow

After modifying `crates/*/src/` or `crates/*/tests/`:

1. Run `bash scripts/coverage-imp-rust.sh --changed` from the workbench root.
2. Include the printed coverage summary in the task response.
3. If changed files or their crate are below `line_target`, add integration tests under `crates/<crate>/tests/` (public API only — see [rust-style.md](./rust-style.md)).
4. Prefer one batched coverage run over many separate `run-in-imp-rust.sh` invocations.

Generated artifacts (`coverage.json`, `lcov.info`, `target/llvm-cov/`) are gitignored — do not commit them.

## Baseline

<!-- Updated by scripts/coverage-imp-rust.sh baseline run -->
**Last measured:** 2026-08-30

**Workspace line coverage:** 100.0%

**Per crate:**

| Crate | Lines | Notes |
| ----- | ----- | ----- |
| imp_collection_transforms | 100.0% | |
| imp_core_types | 100.0% | |
| imp_pathing | 100.0% | |
| imp_registry | 100.0% | |


