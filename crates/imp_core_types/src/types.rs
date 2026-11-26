//! Type helpers. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use serde::{Deserialize, Serialize};

pub type TypeParamId = String;

/// Opaque type parameter on a polymorphic catalog entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeParam {
    pub id: TypeParamId,
    /// A type argument must be assignable to every bound (typically a named union).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounds: Vec<Type>,
}

/// Imp type — concrete nominal type (or a reference to a type definition), type-variable
/// reference, or union.
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
    /// A value of exactly one of the member types (no runtime tag says which).
    Union {
        union: Vec<Type>,
    },
}

impl Type {
    /// Direct child types (concrete args or union members).
    pub fn children(&self) -> &[Type] {
        match self {
            Type::Var { .. } => &[],
            Type::Concrete { args, .. } => args,
            Type::Union { union } => union,
        }
    }

    /// Rebuilds the type with each direct child replaced by `map(child)`.
    pub fn map_children(&self, map: impl FnMut(&Type) -> Type) -> Type {
        match self {
            Type::Var { .. } => self.clone(),
            Type::Concrete { id, args } => Type::Concrete {
                id: id.clone(),
                args: args.iter().map(map).collect(),
            },
            Type::Union { union } => Type::Union {
                union: union.iter().map(map).collect(),
            },
        }
    }
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

pub fn is_union(ty: &Type) -> bool {
    matches!(ty, Type::Union { .. })
}

pub fn concrete_type(id: impl Into<String>, args: Vec<Type>) -> Type {
    Type::Concrete {
        id: id.into(),
        args,
    }
}

pub fn type_var(param: impl Into<String>) -> Type {
    Type::Var {
        param: param.into(),
    }
}

pub fn union_of(members: Vec<Type>) -> Type {
    Type::Union { union: members }
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
