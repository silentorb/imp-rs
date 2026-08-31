//! Substitution helpers.

use imp_core_types::{concrete_type, is_type_var, NodeId, SignalType, TypeParamId};
use std::collections::BTreeMap;

pub type Substitution = BTreeMap<TypeParamId, SignalType>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckError {
    pub node_id: Option<String>,
    pub port_id: Option<String>,
    pub message: String,
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
