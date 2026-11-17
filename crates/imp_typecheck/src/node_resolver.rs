//! Optional host resolution for edge endpoints absent from `Graph.nodes`.

use imp_core_types::{Node, NodeDefinition, NodeId};

/// Synthetic endpoint view supplied by a host when `Graph.nodes` has no entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEndpoint {
    /// Identity / `typeArgs` for port scoping (not inserted into the graph).
    pub node: Node,
    /// Port templates used like a selected local overload.
    pub definition: NodeDefinition,
}

/// Host lookup for NodeIds referenced by edges but missing from the local map.
pub trait NodeResolver {
    fn resolve_endpoint(&self, node_id: &NodeId) -> Option<ResolvedEndpoint>;
}

impl<F> NodeResolver for F
where
    F: Fn(&NodeId) -> Option<ResolvedEndpoint>,
{
    fn resolve_endpoint(&self, node_id: &NodeId) -> Option<ResolvedEndpoint> {
        self(node_id)
    }
}
