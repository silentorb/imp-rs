//! Dynamic Imp graph execution runtime.
//!
//! Demand-driven evaluation of collection/path graphs via a read-only host.
//! Spec: imp-spec/docs/packages/imp-execution/execution.md
//!
//! Performance tiers (future): tree-walking interpreter (v1), optional memoization,
//! bytecode VM, and buffer-oriented hosts for DSP.

mod error;
mod eval;
mod execute;
mod host;
mod resolve;

pub use error::ExecutionError;
pub use execute::{execute_graph, ExecuteGraphOptions};
pub use imp_graph_resolve::ExecutionProgram;
pub use host::{
    ExecutionCapabilities, ExecutionHost, ExecutionResult, ExecutionRow,
    DEFAULT_EXECUTION_CAPABILITIES,
};
