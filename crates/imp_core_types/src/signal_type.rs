//! Signal type helpers. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use serde::{Deserialize, Serialize};

pub type TypeParamId = String;

/// Opaque type parameter on a polymorphic catalog entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeParam {
    pub id: TypeParamId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounds: Vec<String>,
}

/// Imp signal type — concrete nominal type or type-variable reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalType {
    Var {
        param: TypeParamId,
    },
    Concrete {
        id: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<SignalType>,
    },
}

pub fn type_param(id: impl Into<String>) -> TypeParam {
    TypeParam {
        id: id.into(),
        bounds: Vec::new(),
    }
}

pub fn is_concrete(signal_type: &SignalType) -> bool {
    matches!(signal_type, SignalType::Concrete { .. })
}

pub fn is_type_var(signal_type: &SignalType) -> bool {
    matches!(signal_type, SignalType::Var { .. })
}

pub fn concrete_type(id: impl Into<String>, args: Vec<SignalType>) -> SignalType {
    let id = id.into();
    if args.is_empty() {
        SignalType::Concrete { id, args: vec![] }
    } else {
        SignalType::Concrete { id, args }
    }
}

pub fn type_var(param: impl Into<String>) -> SignalType {
    SignalType::Var {
        param: param.into(),
    }
}

pub fn collection_of(element: SignalType) -> SignalType {
    concrete_type("collection", vec![element])
}

pub fn list_of(element: SignalType) -> SignalType {
    concrete_type("list", vec![element])
}
