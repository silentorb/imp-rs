//! Signal-type unification.

use imp_core_types::SignalType;
use thiserror::Error;

use crate::substitution::{
    any_signal, apply_substitution, bind_all_type_vars, contains_type_var, is_any_type, Substitution,
};

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct UnifyError {
    pub message: String,
}

pub fn unify(
    left: &SignalType,
    right: &SignalType,
    subst: &Substitution,
) -> Result<Substitution, UnifyError> {
    let a = apply_substitution(left, subst);
    let b = apply_substitution(right, subst);

    if is_any_type(&a) {
        if contains_type_var(&b) {
            return Ok(bind_all_type_vars(&b, &any_signal(), subst));
        }
        return Ok(subst.clone());
    }
    if is_any_type(&b) {
        if contains_type_var(&a) {
            return Ok(bind_all_type_vars(&a, &any_signal(), subst));
        }
        return Ok(subst.clone());
    }

    match (&a, &b) {
        (SignalType::Var { param: left_param }, SignalType::Var { param: right_param }) => {
            if left_param == right_param {
                return Ok(subst.clone());
            }
            if let Some(existing) = subst.get(left_param) {
                return unify(existing, &b, subst);
            }
            if let Some(existing) = subst.get(right_param) {
                return unify(&a, existing, subst);
            }
            let mut next = subst.clone();
            next.insert(right_param.clone(), SignalType::Var {
                param: left_param.clone(),
            });
            Ok(next)
        }
        (SignalType::Var { param }, other) => unify_var(param, other, subst),
        (other, SignalType::Var { param }) => unify_var(param, other, subst),
        (
            SignalType::Concrete {
                id: left_id,
                args: left_args,
            },
            SignalType::Concrete {
                id: right_id,
                args: right_args,
            },
        ) => {
            if left_id != right_id {
                return Err(UnifyError {
                    message: format!(
                        "cannot unify {} with {}",
                        format_signal_type(&a),
                        format_signal_type(&b)
                    ),
                });
            }
            if left_args.len() != right_args.len() {
                return Err(UnifyError {
                    message: format!(
                        "arity mismatch for {left_id}: {} vs {}",
                        left_args.len(),
                        right_args.len()
                    ),
                });
            }
            let mut current = subst.clone();
            for (left_arg, right_arg) in left_args.iter().zip(right_args.iter()) {
                current = unify(left_arg, right_arg, &current)?;
            }
            Ok(current)
        }
    }
}

fn unify_var(
    param: &str,
    other: &SignalType,
    subst: &Substitution,
) -> Result<Substitution, UnifyError> {
    let resolved = apply_substitution(other, subst);
    if let SignalType::Var { param: other_param } = &resolved {
        if other_param == param {
            return Ok(subst.clone());
        }
        return unify(
            &SignalType::Var {
                param: param.into(),
            },
            &resolved,
            subst,
        );
    }

    if let Some(existing) = subst.get(param) {
        return unify(existing, &resolved, subst);
    }

    let mut next = subst.clone();
    next.insert(param.into(), resolved);
    Ok(next)
}

pub fn format_signal_type(signal_type: &SignalType) -> String {
    match signal_type {
        SignalType::Var { param } => param.clone(),
        SignalType::Concrete { id, args } if args.is_empty() => id.clone(),
        SignalType::Concrete { id, args } => {
            let rendered = args
                .iter()
                .map(format_signal_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{id}<{rendered}>")
        }
    }
}
