//! Node type libraries. Spec: imp-spec/docs/packages/imp-core-types/node-libraries.md

use crate::implementation::NodeImplementation;
use crate::{NodeTypeId, Ports, TypeParam};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Catalog entry for a node type — port templates for a `NodeTypeId`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeType {
    pub id: NodeTypeId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TypeParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<NodeImplementation>,
    pub inputs: Ports,
    pub outputs: Ports,
}

/// Named declarative collection of `NodeType`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeLibrary {
    pub id: String,
    pub types: BTreeMap<NodeTypeId, NodeType>,
}
