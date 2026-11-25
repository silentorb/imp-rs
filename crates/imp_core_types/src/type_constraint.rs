//! Type constraint libraries. Spec: imp-spec/docs/packages/imp-core-types/type-constraints.md

use crate::Type;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type TypeConstraintId = String;

/// Catalog entry listing nominal type patterns that satisfy a constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeConstraint {
    pub id: TypeConstraintId,
    pub members: Vec<Type>,
}

/// Named declarative collection of `TypeConstraint`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeConstraintLibrary {
    pub id: String,
    pub constraints: BTreeMap<TypeConstraintId, TypeConstraint>,
}
