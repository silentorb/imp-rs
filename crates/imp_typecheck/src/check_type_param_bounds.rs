//! Type-parameter bound checking. Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

use imp_core_types::{resolved_nodes, GraphSource, SignalType, TypeConstraint};
use imp_registry::{get_type_constraint, Registry};

use crate::graph_analysis::{resolve_selections, SelectedDefinitions};
use crate::resolve_implementation::signal_type_matches;
use crate::resolve_type_args::resolve_type_args;
use crate::substitution::{is_any_type, rigid_parts, TypeCheckError};
use crate::unify::format_signal_type;

fn satisfies_constraint(type_arg: &SignalType, constraint: &TypeConstraint) -> bool {
    if let Some((_, bounds)) = rigid_parts(type_arg) {
        return bounds.contains(&constraint.id.as_str());
    }
    constraint.members.iter().any(|member| {
        let SignalType::Concrete { .. } = member else {
            return false;
        };
        signal_type_matches(member, type_arg)
    })
}

pub fn check_type_param_bounds<'a>(
    graph: impl Into<GraphSource<'a>>,
    registry: &Registry,
    selected: Option<&SelectedDefinitions>,
) -> Vec<TypeCheckError> {
    let source: GraphSource<'a> = graph.into();
    let mut errors = Vec::new();
    let owned;
    let selections = match selected {
        Some(map) => map,
        None => {
            owned = resolve_selections(source, registry).selected;
            &owned
        }
    };

    for node in &resolved_nodes(source) {
        let Some(definition) = selections.get(&node.id) else {
            continue;
        };
        if definition.type_params.is_empty() {
            continue;
        }

        let Some(type_args) = resolve_type_args(source, &node.id, registry, Some(selections)) else {
            continue;
        };

        for (index, param) in definition.type_params.iter().enumerate() {
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
                    errors.push(TypeCheckError::at_node(
                        node.id.clone(),
                        format!("unknown type constraint \"{bound_id}\""),
                    ));
                    continue;
                };

                if !satisfies_constraint(type_arg, constraint)
                    && !rigid_implies(type_arg, constraint, registry)
                {
                    errors.push(TypeCheckError::at_node(
                        node.id.clone(),
                        format!(
                            "type argument {} does not satisfy bound \"{bound_id}\"",
                            format_signal_type(type_arg)
                        ),
                    ));
                }
            }
        }
    }

    errors
}

/// A rigid type also satisfies `constraint` when every member of one of its own bounds does.
fn rigid_implies(type_arg: &SignalType, constraint: &TypeConstraint, registry: &Registry) -> bool {
    let Some((_, bounds)) = rigid_parts(type_arg) else {
        return false;
    };
    bounds.iter().any(|bound| {
        get_type_constraint(registry, bound).is_some_and(|own| {
            !own.members.is_empty()
                && own
                    .members
                    .iter()
                    .all(|member| satisfies_constraint(member, constraint))
        })
    })
}
