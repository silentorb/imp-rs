//! Execution error types.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ExecutionError {
    #[error("{0}")]
    Message(String),

    #[error("Cycle detected while executing collection at node \"{0}\"")]
    CycleDetected(String),

    #[error("Multiple edges target {node}.{port}; expected at most one")]
    DuplicateEdgeTarget { node: String, port: String },

    #[error("Unknown node \"{0}\"")]
    UnknownNode(String),

    #[error("Unknown node type \"{0}\"")]
    UnknownNodeType(String),

    #[error("Required input \"{node}.{port}\" is unsatisfied (no edge, local value, or default)")]
    UnsatisfiedInput { node: String, port: String },

    #[error("ExecutionHost must grant read capability")]
    ReadCapabilityRequired,

    #[error("imp-execution v1 does not support write capabilities")]
    WriteCapabilityRejected,

    #[error("Node type \"search\" requires ExecutionHost.text_search")]
    MissingTextSearch,

    #[error("Graph has no \"{0}\" boundary node")]
    NoBoundaryNode(String),

    #[error("Graph has {count} \"{kind}\" nodes; pass explicit source/sink")]
    MultipleBoundaryNodes { kind: String, count: usize },

    #[error("Sink \"{node}.{port}\" must be a wired collection")]
    UnwiredSink { node: String, port: String },
}

impl ExecutionError {
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}
