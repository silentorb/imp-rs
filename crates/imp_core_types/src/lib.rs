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
    collection_of, concrete_type, is_concrete, is_type_var, list_of, set_of, type_param, type_var,
    SignalType, TypeParam, TypeParamId,
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

/// One port template: identity + signal type + optional default + optional variadic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    #[serde(rename = "type")]
    pub signal_type: SignalType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<PrimitiveValue>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub variadic: bool,
}

/// Container of port templates — used for NodeDefinition inputs/outputs.
pub type Ports = BTreeMap<PortId, Port>;

/// Local literal values on a node instance, keyed by input PortId.
pub type InputValues = BTreeMap<PortId, PrimitiveValue>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortReference {
    pub node: NodeId,
    pub port: PortId,
}

/// Graph node instance: identity, optional display title, type ref, and local input literals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    /// Optional human-facing label; editors show this when present, else `id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
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
    /// Required and unique among edges into the same variadic target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default)]
    pub nodes: BTreeMap<NodeId, Node>,
    #[serde(default)]
    pub edges: BTreeMap<EdgeId, Edge>,
}

mod library;
pub use library::{
    is_graph_backed_definition, BoundaryBindings, GraphLibrary, NodeDefinition, NodeLibrary, NodeType,
};

mod type_constraint;
pub use type_constraint::{
    TypeConstraint, TypeConstraintId, TypeConstraintLibrary,
};

mod implementation;
pub use implementation::{
    ImplementationCase, ImplementationId, NodeImplementation,
};

mod core_library;
pub use core_library::core_node_library;
