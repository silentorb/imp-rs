//! Type-argument inference for implementation dispatch.

use imp_core_types::{Graph, NodeId, SignalType};
use imp_registry::{get_node_type, Registry};

use crate::graph_analysis::collect_graph_substitution;
use crate::substitution::{apply_substitution, scoped_param};

pub fn resolve_type_args(
    graph: &Graph,
    node_id: &NodeId,
    registry: &Registry,
) -> Option<Vec<SignalType>> {
    let node = graph.nodes.get(node_id)?;
    let node_type = get_node_type(registry, &node.node_type)?;

    if node_type.type_params.is_empty() {
        return Some(vec![]);
    }

    if !node.type_args.is_empty() {
        return Some(node.type_args.clone());
    }

    let subst = collect_graph_substitution(graph, registry);
    let mut resolved = Vec::new();

    for param in &node_type.type_params {
        let scoped = scoped_param(&node.id, &param.id);
        let bound = subst.get(&scoped)?;
        if matches!(bound, SignalType::Var { .. }) {
            return None;
        }
        resolved.push(apply_substitution(bound, &subst));
    }

    Some(resolved)
}
