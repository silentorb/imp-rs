//! Core Imp graph model. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type NodeId = String;
pub type EdgeId = String;
pub type NodeTypeId = String;
pub type PortId = String;
pub type SignalTypeId = String;

mod signal_type;
pub use signal_type::{
    collection_of, concrete_type, is_concrete, is_type_var, type_var, SignalType, TypeParam,
    TypeParamId,
};

/// Literal value for instance inputs or port defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimitiveValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

/// One port template: identity + signal type + optional default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    #[serde(rename = "type")]
    pub signal_type: SignalType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<PrimitiveValue>,
}

/// Container of port templates — used for NodeType inputs/outputs.
pub type Ports = BTreeMap<PortId, Port>;

/// Local literal values on a node instance, keyed by input PortId.
pub type InputValues = BTreeMap<PortId, PrimitiveValue>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortReference {
    pub node: NodeId,
    pub port: PortId,
}

/// Graph node instance: identity, type ref, and local input literals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    #[serde(rename = "type")]
    pub node_type: NodeTypeId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_args: Vec<SignalType>,
    #[serde(default)]
    pub inputs: InputValues,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: PortReference,
    pub to: PortReference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default)]
    pub nodes: BTreeMap<NodeId, Node>,
    #[serde(default)]
    pub edges: BTreeMap<EdgeId, Edge>,
}

mod library;
pub use library::{NodeLibrary, NodeType};

mod graph_type;
pub use graph_type::{GraphType, GraphTypeId, GraphTypeLibrary};

mod implementation;
pub use implementation::{
    ImplementationCase, ImplementationId, NodeImplementation,
};

mod core_library;
pub use core_library::core_node_library;
