//! Type helpers. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use serde::{Deserialize, Serialize};

pub type TypeParamId = String;

/// Opaque type parameter on a polymorphic catalog entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeParam {
    pub id: TypeParamId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounds: Vec<String>,
}

/// Imp type — concrete nominal type or type-variable reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Type {
    Var {
        param: TypeParamId,
    },
    Concrete {
        id: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<Type>,
    },
}

pub fn type_param(id: impl Into<String>) -> TypeParam {
    TypeParam {
        id: id.into(),
        bounds: Vec::new(),
    }
}

pub fn is_concrete(ty: &Type) -> bool {
    matches!(ty, Type::Concrete { .. })
}

pub fn is_type_var(ty: &Type) -> bool {
    matches!(ty, Type::Var { .. })
}

pub fn concrete_type(id: impl Into<String>, args: Vec<Type>) -> Type {
    let id = id.into();
    if args.is_empty() {
        Type::Concrete { id, args: vec![] }
    } else {
        Type::Concrete { id, args }
    }
}

pub fn type_var(param: impl Into<String>) -> Type {
    Type::Var {
        param: param.into(),
    }
}

pub fn collection_of(element: Type) -> Type {
    concrete_type("collection", vec![element])
}

pub fn list_of(element: Type) -> Type {
    concrete_type("list", vec![element])
}

pub fn set_of(element: Type) -> Type {
    concrete_type("set", vec![element])
}
