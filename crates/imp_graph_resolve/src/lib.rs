//! Late nested graph resolution.
//! Spec: imp-spec/docs/packages/imp-graph-resolve/resolve.md

mod resolve;

pub use resolve::{
    build_execution_program, flatten_graph, is_graph_backed_node_type, subgraph_key,
    ExecutionProgram,
};

pub type SubgraphKey = String;
