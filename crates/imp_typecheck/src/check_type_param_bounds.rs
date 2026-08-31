//! Type-parameter bound checking. Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

use imp_core_types::{Graph, SignalType, TypeConstraint};
use imp_registry::{get_node_type, get_type_constraint, Registry};

use crate::resolve_implementation::signal_type_matches;
use crate::resolve_type_args::resolve_type_args;
use crate::substitution::{is_any_type, TypeCheckError};
use crate::unify::format_signal_type;

fn satisfies_constraint(type_arg: &SignalType, constraint: &TypeConstraint) -> bool {
    constraint.members.iter().any(|member| {
        let SignalType::Concrete { .. } = member else {
            return false;
        };
        signal_type_matches(member, type_arg)
    })
}

pub fn check_type_param_bounds(graph: &Graph, registry: &Registry) -> Vec<TypeCheckError> {
    let mut errors = Vec::new();

    for node in graph.nodes.values() {
        let Some(node_type) = get_node_type(registry, &node.node_type) else {
            continue;
        };
        if node_type.type_params.is_empty() {
            continue;
        }

        let Some(type_args) = resolve_type_args(graph, &node.id, registry) else {
            continue;
        };

        for (index, param) in node_type.type_params.iter().enumerate() {
            if param.bounds.is_empty() {
                continue;
            }

            let Some(type_arg) = type_args.get(index) else {
                continue;
            };
            let SignalType::Concrete { .. } = type_arg else {
                continue;
            };

            if is_any_type(type_arg) {
                continue;
            }

            for bound_id in &param.bounds {
                let Some(constraint) = get_type_constraint(registry, bound_id) else {
                    errors.push(TypeCheckError {
                        node_id: Some(node.id.clone()),
                        port_id: None,
                        message: format!("unknown type constraint \"{bound_id}\""),
                    });
                    continue;
                };

                if !satisfies_constraint(type_arg, constraint) {
                    errors.push(TypeCheckError {
                        node_id: Some(node.id.clone()),
                        port_id: None,
                        message: format!(
                            "type argument {} does not satisfy bound \"{bound_id}\"",
                            format_signal_type(type_arg)
                        ),
                    });
                }
            }
        }
    }

    errors
}
