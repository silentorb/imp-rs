//! Type-argument inference for implementation dispatch.

use imp_core_types::{Graph, NodeId, SignalType};
use imp_registry::{get_node_definitions, Registry};

use crate::graph_analysis::{resolve_selections, SelectedDefinitions};
use crate::substitution::{apply_substitution, scoped_param};

pub fn resolve_type_args(
    graph: &Graph,
    node_id: &NodeId,
    registry: &Registry,
    selected: Option<&SelectedDefinitions>,
) -> Option<Vec<SignalType>> {
    let node = graph.nodes.get(node_id)?;
    let overloads = get_node_definitions(registry, &node.node_type);
    let owned;
    let selections = match selected {
        Some(map) => map,
        None => {
            owned = resolve_selections(graph, registry).selected;
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

    let subst = resolve_selections(graph, registry).subst;
    let mut resolved = Vec::new();

    for param in &definition.type_params {
        let scoped = scoped_param(&node.id, &param.id);
        let bound = subst.get(&scoped)?;
        if matches!(bound, SignalType::Var { .. }) {
            return None;
        }
        resolved.push(apply_substitution(bound, &subst));
    }

    Some(resolved)
}
