//! Graph type libraries. Spec: imp-spec/docs/packages/imp-core-types/graph-types.md

use crate::{Ports, TypeParam};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type GraphTypeId = String;

/// Catalog entry for a reusable subgraph interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphType {
    pub id: GraphTypeId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TypeParam>,
    pub inputs: Ports,
    pub outputs: Ports,
}

/// Named declarative collection of `GraphType`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphTypeLibrary {
    pub id: String,
    pub types: BTreeMap<GraphTypeId, GraphType>,
}
