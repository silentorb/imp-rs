//! Edge assignability (directional). Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

use imp_core_types::Type;

use crate::subtype::is_subtype_id;
use crate::substitution::{apply_substitution, Substitution};
use crate::unify::{format_type, unify, UnifyError};

/// Succeeds when a value of `source` may flow into a port of type `target`.
pub fn assignable(
    source: &Type,
    target: &Type,
    subst: &Substitution,
) -> Result<Substitution, UnifyError> {
    match unify(source, target, subst) {
        Ok(next) => Ok(next),
        Err(unify_err) => {
            let a = apply_substitution(source, subst);
            let b = apply_substitution(target, subst);

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
                return Err(UnifyError {
                    message: format!(
                        "cannot assign {} to {}",
                        format_type(&a),
                        format_type(&b)
                    ),
                });
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
    }
}
