//! Boot-time Imp sync graph wire-up for `mir.sync.store`.

use crate::store::StoreRegistry;
use imp_core_types::{Edge, Graph, Node, PortReference, PrimitiveValue};
use std::collections::{BTreeMap, VecDeque};
use thiserror::Error;

/// Imp node type id for binding a registry store into the sync graph.
pub const MIR_SYNC_STORE: &str = "mir.sync.store";

/// Ports used by `mir.sync.store`.
pub const PORT_STORE_ID: &str = "storeId";
pub const PORT_OBSERVE_OUT: &str = "observeOut";
pub const PORT_OBSERVE_IN: &str = "observeIn";

/// One observe edge from a source store to a sink store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserveEdge {
    pub source_store_id: String,
    pub sink_store_id: String,
    pub from_node: String,
    pub to_node: String,
}

/// Validated boot-time wiring derived from an Imp sync graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiredSync {
    pub edges: Vec<ObserveEdge>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WireError {
    #[error("unknown sync node type `{node_type}` on node `{node_id}`")]
    UnknownNodeType { node_id: String, node_type: String },
    #[error("node `{node_id}` missing string input `{port}`")]
    MissingStoreId { node_id: String, port: String },
    #[error("sync graph references unknown store id `{store_id}` (node `{node_id}`)")]
    UnknownStore { node_id: String, store_id: String },
    #[error("store `{store_id}` lacks capability required by node `{node_id}` port `{port}`")]
    MissingCapability {
        node_id: String,
        store_id: String,
        port: String,
    },
    #[error("observe edge `{edge_id}` has unknown endpoint node")]
    UnknownEndpoint { edge_id: String },
    #[error("observe edge `{edge_id}` ports must be observeOut → observeIn")]
    InvalidPorts { edge_id: String },
    #[error("sync graph contains a cycle")]
    Cycle,
}

/// Validate and extract observe edges from a Mir sync Imp graph.
///
/// Boot-only: returns the edge list for the host runtime. Rejects unknown node types,
/// missing stores, capability mismatches, and cycles.
pub fn wire_sync_graph(graph: &Graph, registry: &StoreRegistry) -> Result<WiredSync, WireError> {
    let mut node_store: BTreeMap<&str, String> = BTreeMap::new();
    let mut needs_out: BTreeMap<&str, bool> = BTreeMap::new();
    let mut needs_in: BTreeMap<&str, bool> = BTreeMap::new();

    for (node_id, node) in &graph.nodes {
        if node.node_type != MIR_SYNC_STORE {
            return Err(WireError::UnknownNodeType {
                node_id: node_id.clone(),
                node_type: node.node_type.clone(),
            });
        }
        let store_id = match node.inputs.get(PORT_STORE_ID) {
            Some(PrimitiveValue::String(s)) if !s.is_empty() => s.clone(),
            _ => {
                return Err(WireError::MissingStoreId {
                    node_id: node_id.clone(),
                    port: PORT_STORE_ID.to_string(),
                })
            }
        };
        if !registry.contains(&store_id) {
            return Err(WireError::UnknownStore {
                node_id: node_id.clone(),
                store_id,
            });
        }
        node_store.insert(node_id.as_str(), store_id);
        needs_out.insert(node_id.as_str(), false);
        needs_in.insert(node_id.as_str(), false);
    }

    let mut edges = Vec::new();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for (edge_id, edge) in &graph.edges {
        if edge.from.port != PORT_OBSERVE_OUT || edge.to.port != PORT_OBSERVE_IN {
            return Err(WireError::InvalidPorts {
                edge_id: edge_id.clone(),
            });
        }
        let from_node = edge.from.node.as_str();
        let to_node = edge.to.node.as_str();
        let source_store_id = node_store
            .get(from_node)
            .ok_or_else(|| WireError::UnknownEndpoint {
                edge_id: edge_id.clone(),
            })?
            .clone();
        let sink_store_id = node_store
            .get(to_node)
            .ok_or_else(|| WireError::UnknownEndpoint {
                edge_id: edge_id.clone(),
            })?
            .clone();

        *needs_out.get_mut(from_node).expect("node present") = true;
        *needs_in.get_mut(to_node).expect("node present") = true;

        adj.entry(from_node).or_default().push(to_node);
        edges.push(ObserveEdge {
            source_store_id,
            sink_store_id,
            from_node: from_node.to_string(),
            to_node: to_node.to_string(),
        });
    }

    for (node_id, store_id) in &node_store {
        let caps = registry
            .get(store_id)
            .map(|s| s.capabilities())
            .unwrap_or_default();
        if needs_out[node_id] && !caps.observe_source {
            return Err(WireError::MissingCapability {
                node_id: (*node_id).to_string(),
                store_id: store_id.clone(),
                port: PORT_OBSERVE_OUT.to_string(),
            });
        }
        if needs_in[node_id] && !caps.observe_sink {
            return Err(WireError::MissingCapability {
                node_id: (*node_id).to_string(),
                store_id: store_id.clone(),
                port: PORT_OBSERVE_IN.to_string(),
            });
        }
    }

    if has_cycle(graph.nodes.keys().map(|k| k.as_str()), &adj) {
        return Err(WireError::Cycle);
    }

    Ok(WiredSync { edges })
}

fn has_cycle<'a>(nodes: impl Iterator<Item = &'a str>, adj: &BTreeMap<&str, Vec<&str>>) -> bool {
    let mut indegree: BTreeMap<&str, usize> = BTreeMap::new();
    for n in nodes {
        indegree.entry(n).or_insert(0);
    }
    for targets in adj.values() {
        for t in targets {
            *indegree.entry(t).or_insert(0) += 1;
        }
    }
    let mut queue: VecDeque<&str> = indegree
        .iter()
        .filter_map(|(n, d)| if *d == 0 { Some(*n) } else { None })
        .collect();
    let mut visited = 0usize;
    let total = indegree.len();
    while let Some(n) = queue.pop_front() {
        visited += 1;
        if let Some(nexts) = adj.get(n) {
            for t in nexts {
                if let Some(d) = indegree.get_mut(t) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(t);
                    }
                }
            }
        }
    }
    visited != total
}

/// Build the default Miro Imp sync graph: text observeOut → sqlite observeIn.
pub fn default_miro_graph(text_store_id: &str, sqlite_store_id: &str) -> Graph {
    let mut nodes = BTreeMap::new();
    nodes.insert(
        "authoring".to_string(),
        Node {
            id: "authoring".to_string(),
            title: None,
            node_type: MIR_SYNC_STORE.to_string(),
            type_args: Vec::new(),
            inputs: BTreeMap::from([(
                PORT_STORE_ID.to_string(),
                PrimitiveValue::String(text_store_id.to_string()),
            )]),
        },
    );
    nodes.insert(
        "projection".to_string(),
        Node {
            id: "projection".to_string(),
            title: None,
            node_type: MIR_SYNC_STORE.to_string(),
            type_args: Vec::new(),
            inputs: BTreeMap::from([(
                PORT_STORE_ID.to_string(),
                PrimitiveValue::String(sqlite_store_id.to_string()),
            )]),
        },
    );
    let mut edges = BTreeMap::new();
    edges.insert(
        "obs".to_string(),
        Edge {
            from: PortReference {
                node: "authoring".to_string(),
                port: PORT_OBSERVE_OUT.to_string(),
            },
            to: PortReference {
                node: "projection".to_string(),
                port: PORT_OBSERVE_IN.to_string(),
            },
        
        order: None,
        },
    );
    Graph { nodes, edges }
}
