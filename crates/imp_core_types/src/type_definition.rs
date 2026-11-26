//! Type definitions (named types). Spec: imp-spec/docs/packages/imp-core-types/type-definitions.md

use crate::{Type, TypeId, TypeParam};
use serde::{Deserialize, Serialize};

/// `id<params> = type`: a transparent alias, e.g. `Audio = audio_mono | audio_stereo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinition {
    pub id: TypeId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<TypeParam>,
    #[serde(rename = "type")]
    pub ty: Type,
}
