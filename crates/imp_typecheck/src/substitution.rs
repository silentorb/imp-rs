//! Substitution helpers.

use imp_core_types::{concrete_type, is_type_var, NodeId, Type, TypeParamId};
use std::collections::BTreeMap;

pub type Substitution = BTreeMap<TypeParamId, Type>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckError {
    pub node_id: Option<String>,
    pub port_id: Option<String>,
    pub definition_id: Option<String>,
    pub message: String,
}

impl TypeCheckError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            node_id: None,
            port_id: None,
            definition_id: None,
            message: message.into(),
        }
    }

    pub fn at_node(node_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_id: Some(node_id.into()),
            port_id: None,
            definition_id: None,
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
            message: message.into(),
        }
    }

    pub fn for_definition(definition_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            node_id: None,
            port_id: None,
            definition_id: Some(definition_id.into()),
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
        Type::Concrete { id, args } if !args.is_empty() => Type::Concrete {
            id: id.clone(),
            args: args
                .iter()
                .map(|arg| scope_type(arg, node_id))
                .collect(),
        },
        other => other.clone(),
    }
}

pub fn apply_substitution(ty: &Type, subst: &Substitution) -> Type {
    match ty {
        Type::Var { param } => subst
            .get(param)
            .map(|resolved| apply_substitution(resolved, subst))
            .unwrap_or_else(|| ty.clone()),
        Type::Concrete { id, args } if !args.is_empty() => Type::Concrete {
            id: id.clone(),
            args: args
                .iter()
                .map(|arg| apply_substitution(arg, subst))
                .collect(),
        },
        other => other.clone(),
    }
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
        Type::Concrete { args, .. } => args.iter().any(contains_type_var),
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
        Type::Concrete { args, .. } => {
            for arg in args {
                bind_type_vars_into(arg, binding, subst);
            }
        }
    }
}

pub fn assert_concrete_args(args: &[Type]) -> Result<(), String> {
    for arg in args {
        if is_type_var(arg) || contains_type_var(arg) {
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
pub fn rigid_type(param: &str, bounds: &[String]) -> Type {
    let mut args = vec![concrete_type(param, vec![])];
    args.extend(bounds.iter().map(|bound| concrete_type(bound.as_str(), vec![])));
    Type::Concrete {
        id: RIGID_TYPE_ID.into(),
        args,
    }
}

/// The parameter id and bounds of a rigid type, or `None` for any other type.
pub fn rigid_parts(ty: &Type) -> Option<(&str, Vec<&str>)> {
    let Type::Concrete { id, args } = ty else {
        return None;
    };
    if id != RIGID_TYPE_ID {
        return None;
    }
    let mut ids = args.iter().filter_map(|arg| match arg {
        Type::Concrete { id, .. } => Some(id.as_str()),
        Type::Var { .. } => None,
    });
    let param = ids.next()?;
    Some((param, ids.collect()))
}

/// The declared parameter id of a scoped type variable (`node::A` → `A`).
pub fn unscoped_param(param: &str) -> &str {
    param.rsplit("::").next().unwrap_or(param)
}
