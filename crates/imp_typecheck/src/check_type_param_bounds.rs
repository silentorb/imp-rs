//! Type-parameter bound checking. Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

use imp_core_types::{resolved_nodes, GraphSource};
use imp_registry::Registry;

use crate::assignable::assignable;
use crate::graph_analysis::{resolve_selections, SelectedDefinitions};
use crate::normalize::with_prepared;
use crate::resolve_type_args::resolve_type_args;
use crate::substitution::{Substitution, TypeCheckError};
use crate::unify::format_type;

/// Every resolved type argument must be assignable to each of its parameter's bounds (with an
/// empty substitution). `any` satisfies every bound; a rigid parameter satisfies a bound when one
/// of its own bounds is assignable to it.
pub fn check_type_param_bounds<'a>(
    graph: impl Into<GraphSource<'a>>,
    registry: &Registry,
    selected: Option<&SelectedDefinitions>,
) -> Vec<TypeCheckError> {
    with_prepared(graph.into(), registry, |source, registry| {
        check_prepared(source, registry, selected)
    })
}

fn check_prepared(
    source: GraphSource<'_>,
    registry: &Registry,
    selected: Option<&SelectedDefinitions>,
) -> Vec<TypeCheckError> {
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

        let Some(type_args) = resolve_type_args(source, &node.id, registry, Some(selections))
        else {
            continue;
        };

        for (index, param) in definition.type_params.iter().enumerate() {
            let Some(type_arg) = type_args.get(index) else {
                continue;
            };
            for bound in &param.bounds {
                if assignable(type_arg, bound, &Substitution::new()).is_err() {
                    errors.push(TypeCheckError::at_node(
                        node.id.clone(),
                        format!(
                            "type argument {} does not satisfy bound {}",
                            format_type(type_arg),
                            format_type(bound)
                        ),
                    ));
                }
            }
        }
    }

    errors
}
