//! Substitution helpers.

use imp_core_types::{concrete_type, NodeId, Type, TypeParamId};
use std::collections::BTreeMap;

pub type Substitution = BTreeMap<TypeParamId, Type>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckError {
    pub node_id: Option<String>,
    pub port_id: Option<String>,
    pub definition_id: Option<String>,
    /// Set for type-definition errors.
    pub type_id: Option<String>,
    pub message: String,
}

impl TypeCheckError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            node_id: None,
            port_id: None,
            definition_id: None,
            type_id: None,
            message: message.into(),
        }
    }

    pub fn at_node(node_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_id: Some(node_id.into()),
            port_id: None,
            definition_id: None,
            type_id: None,
            message: message.into(),
        }
    }

    pub fn at_port(
        node_id: impl Into<String>,
        port_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            node_id: Some(node_id.into()),
            port_id: Some(port_id.into()),
            definition_id: None,
            type_id: None,
            message: message.into(),
        }
    }

    pub fn for_definition(definition_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_id: None,
            port_id: None,
            definition_id: Some(definition_id.into()),
            type_id: None,
            message: message.into(),
        }
    }

    pub fn for_type(type_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_id: None,
            port_id: None,
            definition_id: None,
            type_id: Some(type_id.into()),
            message: message.into(),
        }
    }
}

pub fn scoped_param(node_id: &NodeId, param: &TypeParamId) -> TypeParamId {
    format!("{node_id}::{param}")
}

pub fn scope_type(ty: &Type, node_id: &NodeId) -> Type {
    match ty {
        Type::Var { param } => Type::Var {
            param: scoped_param(node_id, param),
        },
        other => other.map_children(|child| scope_type(child, node_id)),
    }
}

pub fn apply_substitution(ty: &Type, subst: &Substitution) -> Type {
    match ty {
        Type::Var { param } => subst
            .get(param)
            .map(|resolved| apply_substitution(resolved, subst))
            .unwrap_or_else(|| ty.clone()),
        Type::Union { union } => canonical_union(
            union
                .iter()
                .map(|member| apply_substitution(member, subst))
                .collect(),
        ),
        other => other.map_children(|child| apply_substitution(child, subst)),
    }
}

/// Canonical union of `members`: nested unions flatten, `any` absorbs, duplicates drop (first
/// occurrence wins), and a one-member union collapses to that member.
pub fn canonical_union(members: Vec<Type>) -> Type {
    let mut flat: Vec<Type> = Vec::new();
    for member in members {
        let parts = match member {
            Type::Union { union } => union,
            other => vec![other],
        };
        for part in parts {
            if is_any_type(&part) {
                return any_signal();
            }
            if !flat.contains(&part) {
                flat.push(part);
            }
        }
    }
    if flat.len() == 1 {
        return flat.remove(0);
    }
    Type::Union { union: flat }
}

pub fn is_any_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Concrete { id, args }
            if id == "any" && args.is_empty()
    )
}

pub fn contains_type_var(ty: &Type) -> bool {
    match ty {
        Type::Var { .. } => true,
        other => other.children().iter().any(contains_type_var),
    }
}

pub fn bind_all_type_vars(
    ty: &Type,
    binding: &Type,
    subst: &Substitution,
) -> Substitution {
    let mut next = subst.clone();
    bind_type_vars_into(ty, binding, &mut next);
    next
}

fn bind_type_vars_into(ty: &Type, binding: &Type, subst: &mut Substitution) {
    match ty {
        Type::Var { param } => {
            if !subst.contains_key(param) {
                subst.insert(param.clone(), binding.clone());
            }
        }
        other => {
            for child in other.children() {
                bind_type_vars_into(child, binding, subst);
            }
        }
    }
}

pub fn assert_concrete_args(args: &[Type]) -> Result<(), String> {
    for arg in args {
        if contains_type_var(arg) {
            return Err("typeArgs must be concrete types".into());
        }
    }
    Ok(())
}

pub fn any_signal() -> Type {
    concrete_type("any", vec![])
}

/// Reserved concrete id for a rigid type parameter while checking a generic graph-backed
/// definition's body. Never serialized; see type-system.md (Generic graph-backed definitions).
pub const RIGID_TYPE_ID: &str = "$rigid";

/// A rigid stand-in for definition type parameter `param` with its declared `bounds`.
///
/// Encoded as `$rigid<param, bound…>` so that it unifies only with itself.
pub fn rigid_type(param: &str, bounds: &[Type]) -> Type {
    let mut args = vec![concrete_type(param, vec![])];
    args.extend(bounds.iter().cloned());
    Type::Concrete {
        id: RIGID_TYPE_ID.into(),
        args,
    }
}

/// The parameter id and bounds of a rigid type, or `None` for any other type.
pub fn rigid_parts(ty: &Type) -> Option<(&str, &[Type])> {
    let Type::Concrete { id, args } = ty else {
        return None;
    };
    if id != RIGID_TYPE_ID {
        return None;
    }
    let (first, bounds) = args.split_first()?;
    let Type::Concrete { id: param, .. } = first else {
        return None;
    };
    Some((param.as_str(), bounds))
}

/// The declared parameter id of a scoped type variable (`node::A` → `A`).
pub fn unscoped_param(param: &str) -> &str {
    param.rsplit("::").next().unwrap_or(param)
}
