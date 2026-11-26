//! Edge assignability (directional). Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

use imp_core_types::Type;

use crate::substitution::{
    apply_substitution, is_any_type, rigid_parts, Substitution, RIGID_TYPE_ID,
};
use crate::subtype::is_subtype_id;
use crate::unify::{format_type, unify, UnifyError};

fn cannot_assign(source: &Type, target: &Type) -> UnifyError {
    UnifyError {
        message: format!(
            "cannot assign {} to {}",
            format_type(source),
            format_type(target)
        ),
    }
}

/// Succeeds when a value of `source` may flow into a port of type `target` (both normalized).
///
/// Rule order (type-system.md, Assignability): `any`; union source (every member, or a variable
/// target takes the whole union); rigid source (via its bounds); variable source into a union
/// (binds the whole union); union target (some member, unambiguously); unify; declared subtypes
/// with invariant arguments.
pub fn assignable(
    source: &Type,
    target: &Type,
    subst: &Substitution,
) -> Result<Substitution, UnifyError> {
    let a = apply_substitution(source, subst);
    let b = apply_substitution(target, subst);

    if is_any_type(&a) || is_any_type(&b) {
        return unify(&a, &b, subst);
    }

    if let Type::Union { union } = &a {
        if matches!(b, Type::Var { .. }) {
            return unify(&a, &b, subst);
        }
        let mut current = subst.clone();
        for member in union {
            current = assignable(member, &b, &current).map_err(|_| cannot_assign(&a, &b))?;
        }
        return Ok(current);
    }

    if let Some((_, bounds)) = rigid_parts(&a) {
        let rigid_target = matches!(&b, Type::Concrete { id, .. } if id == RIGID_TYPE_ID);
        if matches!(b, Type::Var { .. }) || rigid_target {
            return unify(&a, &b, subst);
        }
        if bounds
            .iter()
            .any(|bound| assignable(bound, &b, subst).is_ok())
        {
            return Ok(subst.clone());
        }
        return Err(cannot_assign(&a, &b));
    }

    if let Type::Union { union } = &b {
        if matches!(a, Type::Var { .. }) {
            return unify(&a, &b, subst);
        }
        let mut accepted: Option<Substitution> = None;
        for member in union {
            let Ok(next) = assignable(&a, member, subst) else {
                continue;
            };
            match &accepted {
                Some(prior) if *prior != next => {
                    return Err(UnifyError {
                        message: format!(
                            "{} is ambiguous between members of {}; state typeArgs explicitly",
                            format_type(&a),
                            format_type(&b)
                        ),
                    });
                }
                Some(_) => {}
                None => accepted = Some(next),
            }
        }
        return accepted.ok_or_else(|| cannot_assign(&a, &b));
    }

    let unify_err = match unify(&a, &b, subst) {
        Ok(next) => return Ok(next),
        Err(err) => err,
    };

    let (
        Type::Concrete {
            id: source_id,
            args: source_args,
        },
        Type::Concrete {
            id: target_id,
            args: target_args,
        },
    ) = (&a, &b)
    else {
        return Err(unify_err);
    };

    if source_id == target_id || !is_subtype_id(source_id, target_id) {
        return Err(cannot_assign(&a, &b));
    }

    if source_args.len() != target_args.len() {
        return Err(UnifyError {
            message: format!(
                "arity mismatch for {source_id} <: {target_id}: {} vs {}",
                source_args.len(),
                target_args.len()
            ),
        });
    }

    let mut current = subst.clone();
    for (source_arg, target_arg) in source_args.iter().zip(target_args.iter()) {
        current = unify(source_arg, target_arg, &current)?;
    }
    Ok(current)
}
