//! Node implementation dispatch. Spec: imp-spec/docs/packages/imp-core-types/node-libraries.md

use crate::types::Type;
use crate::NodeTypeId;
use serde::{Deserialize, Serialize};

pub type ImplementationId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImplementationCase {
    #[serde(rename = "match")]
    pub match_type: Type,
    pub implementation: ImplementationId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum NodeImplementation {
    Universal { id: ImplementationId },
    Dispatch {
        #[serde(default, rename = "paramIndex", skip_serializing_if = "Option::is_none")]
        param_index: Option<usize>,
        cases: Vec<ImplementationCase>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<ImplementationId>,
    },
}

impl NodeImplementation {
    pub fn universal(id: impl Into<String>) -> Self {
        Self::Universal { id: id.into() }
    }

    pub fn default_for(node_type_id: &NodeTypeId) -> Self {
        Self::universal(node_type_id.clone())
    }
}
