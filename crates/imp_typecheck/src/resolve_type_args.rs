//! Type-argument inference for implementation dispatch.

use imp_core_types::{GraphSource, NodeId, Type};
use imp_registry::{get_node_definitions, Registry};

use crate::graph_analysis::{resolve_selections, SelectedDefinitions};
use crate::normalize::with_prepared;
use crate::substitution::{apply_substitution, contains_type_var, scoped_param};

pub fn resolve_type_args<'a>(
    graph: impl Into<GraphSource<'a>>,
    node_id: &NodeId,
    registry: &Registry,
    selected: Option<&SelectedDefinitions>,
) -> Option<Vec<Type>> {
    with_prepared(graph.into(), registry, |source, registry| {
        resolve_prepared(source, node_id, registry, selected)
    })
}

fn resolve_prepared(
    source: GraphSource<'_>,
    node_id: &NodeId,
    registry: &Registry,
    selected: Option<&SelectedDefinitions>,
) -> Option<Vec<Type>> {
    let node = source.nodes.get_node(node_id)?;
    let overloads = get_node_definitions(registry, &node.node_type);
    let owned;
    let selections = match selected {
        Some(map) => map,
        None => {
            owned = resolve_selections(source, registry).selected;
            &owned
        }
    };
    let definition = selections.get(node_id).cloned().or_else(|| {
        if overloads.len() == 1 {
            Some(overloads[0].clone())
        } else {
            None
        }
    })?;

    if definition.type_params.is_empty() {
        return Some(vec![]);
    }

    if !node.type_args.is_empty() {
        return Some(node.type_args.clone());
    }

    let subst = resolve_selections(source, registry).subst;
    let mut resolved = Vec::new();

    for param in &definition.type_params {
        let scoped = scoped_param(&node.id, &param.id);
        let bound = subst.get(&scoped)?;
        // A parameter bound to another variable is resolved once that variable is.
        let resolved_type = apply_substitution(bound, &subst);
        if contains_type_var(&resolved_type) {
            return None;
        }
        resolved.push(resolved_type);
    }

    Some(resolved)
}
