//! Read-only edge / node providers that graph algorithms consume.
//! Spec: imp-spec/docs/packages/imp-core-types/graph-sources.md

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Edge, EdgeId, Graph, Node, NodeId, PortId};

/// Supplies nodes by id. Whether a node is local or remote is the provider's concern.
pub trait NodeProvider {
    fn get_node(&self, id: &NodeId) -> Option<Cow<'_, Node>>;

    /// Ids this provider can enumerate cheaply; `None` for non-enumerable (e.g. store-backed) providers.
    fn node_ids(&self) -> Option<Vec<NodeId>> {
        None
    }
}

/// Supplies the graph's topology.
pub trait EdgeProvider {
    fn get_edge(&self, id: &EdgeId) -> Option<&Edge>;
    fn edge_entries(&self) -> Vec<(&EdgeId, &Edge)>;
}

/// One node provider + one edge provider — what graph algorithms accept.
#[derive(Clone, Copy)]
pub struct GraphSource<'a> {
    pub nodes: &'a dyn NodeProvider,
    pub edges: &'a dyn EdgeProvider,
}

impl NodeProvider for BTreeMap<NodeId, Node> {
    fn get_node(&self, id: &NodeId) -> Option<Cow<'_, Node>> {
        self.get(id).map(Cow::Borrowed)
    }

    fn node_ids(&self) -> Option<Vec<NodeId>> {
        Some(self.keys().cloned().collect())
    }
}

impl EdgeProvider for BTreeMap<EdgeId, Edge> {
    fn get_edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.get(id)
    }

    fn edge_entries(&self) -> Vec<(&EdgeId, &Edge)> {
        self.iter().collect()
    }
}

/// Graph source over a serialized `Graph` — behaves exactly like the graph itself.
pub fn graph_source(graph: &Graph) -> GraphSource<'_> {
    GraphSource {
        nodes: &graph.nodes,
        edges: &graph.edges,
    }
}

impl<'a> From<&'a Graph> for GraphSource<'a> {
    fn from(graph: &'a Graph) -> Self {
        graph_source(graph)
    }
}

/// Node provider returning the first hit across `providers` in order.
/// `node_ids` is the de-duplicated union of providers that can enumerate (`None` when none can).
pub struct LayeredNodes<'a> {
    pub providers: Vec<&'a dyn NodeProvider>,
}

impl NodeProvider for LayeredNodes<'_> {
    fn get_node(&self, id: &NodeId) -> Option<Cow<'_, Node>> {
        self.providers
            .iter()
            .find_map(|provider| provider.get_node(id))
    }

    fn node_ids(&self) -> Option<Vec<NodeId>> {
        let lists: Vec<Vec<NodeId>> = self
            .providers
            .iter()
            .filter_map(|provider| provider.node_ids())
            .collect();
        if lists.is_empty() {
            return None;
        }
        let mut seen = BTreeSet::new();
        Some(
            lists
                .into_iter()
                .flatten()
                .filter(|id| seen.insert(id.clone()))
                .collect(),
        )
    }
}

/// Membership: edge endpoints (first occurrence, entry order), then enumerable node ids not yet listed.
pub fn graph_node_ids(source: GraphSource<'_>) -> Vec<NodeId> {
    let mut seen = BTreeSet::new();
    let endpoints = source
        .edges
        .edge_entries()
        .into_iter()
        .flat_map(|(_, edge)| [edge.from.node.clone(), edge.to.node.clone()]);
    let listed = source.nodes.node_ids().unwrap_or_default();
    endpoints
        .chain(listed)
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

/// Resolved members of the source, in membership order.
pub fn resolved_nodes<'a>(source: GraphSource<'a>) -> Vec<Cow<'a, Node>> {
    graph_node_ids(source)
        .iter()
        .filter_map(|id| source.nodes.get_node(id))
        .collect()
}

/// Member ids with no node from the provider.
pub fn unresolved_node_ids(source: GraphSource<'_>) -> Vec<NodeId> {
    graph_node_ids(source)
        .into_iter()
        .filter(|id| source.nodes.get_node(id).is_none())
        .collect()
}

/// Edges targeting `(node_id, port_id)`, sorted by `Edge.order` ascending (unset = 0, ties by entry order).
pub fn inbound_edges<'a>(
    source: GraphSource<'a>,
    node_id: &NodeId,
    port_id: &PortId,
) -> Vec<(&'a EdgeId, &'a Edge)> {
    let mut inbound: Vec<_> = source
        .edges
        .edge_entries()
        .into_iter()
        .filter(|(_, edge)| edge.to.node == *node_id && edge.to.port == *port_id)
        .collect();
    inbound.sort_by_key(|(_, edge)| edge.order.unwrap_or(0));
    inbound
}

/// Serialized `Graph` with every resolved member and every edge.
pub fn materialize_graph(source: GraphSource<'_>) -> Graph {
    Graph {
        nodes: resolved_nodes(source)
            .into_iter()
            .map(|node| (node.id.clone(), node.into_owned()))
            .collect(),
        edges: source
            .edges
            .edge_entries()
            .into_iter()
            .map(|(id, edge)| (id.clone(), edge.clone()))
            .collect(),
    }
}
