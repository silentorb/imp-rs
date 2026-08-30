//! Input resolution for imp-execution (aligned with imp-sql).

use crate::error::ExecutionError;
use imp_core_types::{Edge, Graph, Node, NodeId, NodeType, PortId, PortReference, PrimitiveValue};
use imp_registry::{get_node_type, Registry};
use std::collections::BTreeMap;

pub type EdgeTargetKey = (NodeId, PortId);

pub fn edge_target_key(node: &NodeId, port: &PortId) -> EdgeTargetKey {
    (node.clone(), port.clone())
}

pub fn index_edges_by_target(graph: &Graph) -> Result<BTreeMap<EdgeTargetKey, Edge>, ExecutionError> {
    let mut map = BTreeMap::new();
    for edge in graph.edges.values() {
        let key = edge_target_key(&edge.to.node, &edge.to.port);
        if map.contains_key(&key) {
            return Err(ExecutionError::DuplicateEdgeTarget {
                node: edge.to.node.clone(),
                port: edge.to.port.clone(),
            });
        }
        map.insert(key, edge.clone());
    }
    Ok(map)
}

pub fn require_node<'a>(graph: &'a Graph, node_id: &NodeId) -> Result<&'a Node, ExecutionError> {
    graph
        .nodes
        .get(node_id)
        .ok_or_else(|| ExecutionError::UnknownNode(node_id.clone()))
}

pub fn require_node_type<'a>(
    registry: &'a Registry,
    type_id: &str,
) -> Result<&'a NodeType, ExecutionError> {
    get_node_type(registry, type_id).ok_or_else(|| ExecutionError::UnknownNodeType(type_id.into()))
}

pub fn find_sole_boundary_node<'a>(graph: &'a Graph, type_id: &str) -> Result<&'a Node, ExecutionError> {
    let matches: Vec<&Node> = graph
        .nodes
        .values()
        .filter(|n| n.node_type == type_id)
        .collect();
    match matches.len() {
        0 => Err(ExecutionError::NoBoundaryNode(type_id.into())),
        1 => Ok(matches[0]),
        n => Err(ExecutionError::MultipleBoundaryNodes {
            kind: type_id.into(),
            count: n,
        }),
    }
}

pub fn default_source(graph: &Graph) -> Result<PortReference, ExecutionError> {
    let node = find_sole_boundary_node(graph, "input")?;
    Ok(PortReference {
        node: node.id.clone(),
        port: "value".into(),
    })
}

pub fn default_sink(graph: &Graph) -> Result<PortReference, ExecutionError> {
    let node = find_sole_boundary_node(graph, "output")?;
    Ok(PortReference {
        node: node.id.clone(),
        port: "value".into(),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedInput {
    Literal { value: PrimitiveValue },
    Wire { from: PortReference },
}

pub fn resolve_input(
    graph: &Graph,
    registry: &Registry,
    edges_by_target: &BTreeMap<EdgeTargetKey, Edge>,
    node_id: &NodeId,
    port_id: &PortId,
) -> Result<ResolvedInput, ExecutionError> {
    if let Some(edge) = edges_by_target.get(&edge_target_key(node_id, port_id)) {
        return Ok(ResolvedInput::Wire {
            from: edge.from.clone(),
        });
    }

    let node = require_node(graph, node_id)?;
    if let Some(value) = node.inputs.get(port_id) {
        return Ok(ResolvedInput::Literal {
            value: value.clone(),
        });
    }

    let node_type = require_node_type(registry, &node.node_type)?;
    let port = node_type
        .inputs
        .get(port_id)
        .ok_or_else(|| ExecutionError::message(format!(
            "Node \"{node_id}\" (type \"{}\") has no input port \"{port_id}\"",
            node.node_type
        )))?;
    if let Some(default_value) = &port.default_value {
        return Ok(ResolvedInput::Literal {
            value: default_value.clone(),
        });
    }

    Err(ExecutionError::UnsatisfiedInput {
        node: node_id.clone(),
        port: port_id.clone(),
    })
}
