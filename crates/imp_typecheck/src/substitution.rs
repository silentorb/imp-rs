//! Substitution helpers.

use imp_core_types::{concrete_type, is_type_var, NodeId, SignalType, TypeParamId};
use std::collections::BTreeMap;

pub type Substitution = BTreeMap<TypeParamId, SignalType>;

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

pub fn scope_signal_type(signal_type: &SignalType, node_id: &NodeId) -> SignalType {
    match signal_type {
        SignalType::Var { param } => SignalType::Var {
            param: scoped_param(node_id, param),
        },
        SignalType::Concrete { id, args } if !args.is_empty() => SignalType::Concrete {
            id: id.clone(),
            args: args
                .iter()
                .map(|arg| scope_signal_type(arg, node_id))
                .collect(),
        },
        other => other.clone(),
    }
}

pub fn apply_substitution(signal_type: &SignalType, subst: &Substitution) -> SignalType {
    match signal_type {
        SignalType::Var { param } => subst
            .get(param)
            .map(|resolved| apply_substitution(resolved, subst))
            .unwrap_or_else(|| signal_type.clone()),
        SignalType::Concrete { id, args } if !args.is_empty() => SignalType::Concrete {
            id: id.clone(),
            args: args
                .iter()
                .map(|arg| apply_substitution(arg, subst))
                .collect(),
        },
        other => other.clone(),
    }
}

pub fn is_any_type(signal_type: &SignalType) -> bool {
    matches!(
        signal_type,
        SignalType::Concrete { id, args }
            if id == "any" && args.is_empty()
    )
}

pub fn contains_type_var(signal_type: &SignalType) -> bool {
    match signal_type {
        SignalType::Var { .. } => true,
        SignalType::Concrete { args, .. } => args.iter().any(contains_type_var),
    }
}

pub fn bind_all_type_vars(
    signal_type: &SignalType,
    binding: &SignalType,
    subst: &Substitution,
) -> Substitution {
    let mut next = subst.clone();
    bind_type_vars_into(signal_type, binding, &mut next);
    next
}

fn bind_type_vars_into(signal_type: &SignalType, binding: &SignalType, subst: &mut Substitution) {
    match signal_type {
        SignalType::Var { param } => {
            if !subst.contains_key(param) {
                subst.insert(param.clone(), binding.clone());
            }
        }
        SignalType::Concrete { args, .. } => {
            for arg in args {
                bind_type_vars_into(arg, binding, subst);
            }
        }
    }
}

pub fn assert_concrete_args(args: &[SignalType]) -> Result<(), String> {
    for arg in args {
        if is_type_var(arg) || contains_type_var(arg) {
            return Err("typeArgs must be concrete signal types".into());
        }
    }
    Ok(())
}

pub fn any_signal() -> SignalType {
    concrete_type("any", vec![])
}

/// Reserved concrete id for a rigid type parameter while checking a generic graph-backed
/// definition's body. Never serialized; see type-system.md (Generic graph-backed definitions).
pub const RIGID_TYPE_ID: &str = "$rigid";

/// A rigid stand-in for definition type parameter `param` with its declared `bounds`.
///
/// Encoded as `$rigid<param, bound…>` so that it unifies only with itself.
pub fn rigid_type(param: &str, bounds: &[String]) -> SignalType {
    let mut args = vec![concrete_type(param, vec![])];
    args.extend(bounds.iter().map(|bound| concrete_type(bound.as_str(), vec![])));
    SignalType::Concrete {
        id: RIGID_TYPE_ID.into(),
        args,
    }
}

/// The parameter id and bounds of a rigid type, or `None` for any other type.
pub fn rigid_parts(signal_type: &SignalType) -> Option<(&str, Vec<&str>)> {
    let SignalType::Concrete { id, args } = signal_type else {
        return None;
    };
    if id != RIGID_TYPE_ID {
        return None;
    }
    let mut ids = args.iter().filter_map(|arg| match arg {
        SignalType::Concrete { id, .. } => Some(id.as_str()),
        SignalType::Var { .. } => None,
    });
    let param = ids.next()?;
    Some((param, ids.collect()))
}

/// The declared parameter id of a scoped type variable (`node::A` → `A`).
pub fn unscoped_param(param: &str) -> &str {
    param.rsplit("::").next().unwrap_or(param)
}
