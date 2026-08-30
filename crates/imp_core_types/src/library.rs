//! Node type libraries. Spec: imp-spec/docs/packages/imp-core-types/node-libraries.md

use crate::{NodeTypeId, Ports};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Catalog entry for a node type — port templates for a `NodeTypeId`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeType {
    pub id: NodeTypeId,
    pub inputs: Ports,
    pub outputs: Ports,
}

/// Named declarative collection of `NodeType`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeLibrary {
    pub id: String,
    pub types: BTreeMap<NodeTypeId, NodeType>,
}
