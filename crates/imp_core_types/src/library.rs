//! Node definition libraries. Spec: imp-spec/docs/packages/imp-core-types/node-libraries.md

use crate::implementation::NodeImplementation;
use crate::{Graph, NodeId, NodeTypeId, Ports, TypeParam};
use serde::{Deserialize, Serialize};

/// Maps declared external ports to boundary nodes in a graph-backed body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryBindings {
    pub inputs: std::collections::BTreeMap<crate::PortId, NodeId>,
    pub outputs: std::collections::BTreeMap<crate::PortId, NodeId>,
}

/// Catalog entry for a node type — port templates for a `NodeTypeId`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub id: NodeTypeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TypeParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<NodeImplementation>,
    pub inputs: Ports,
    pub outputs: Ports,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Graph>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bindings: Option<BoundaryBindings>,
}

/// Named declarative collection of `NodeDefinition`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeLibrary {
    pub id: String,
    pub definitions: Vec<NodeDefinition>,
}

/// JSON-serializable catalog of graph-backed definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphLibrary {
    pub id: String,
    pub definitions: Vec<NodeDefinition>,
}

pub fn is_graph_backed_definition(def: &NodeDefinition) -> bool {
    def.body.is_some()
}

pub type NodeType = NodeDefinition;
