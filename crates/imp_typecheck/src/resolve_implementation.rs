//! Implementation resolution from type arguments.

use imp_core_types::{
    Graph, ImplementationId, NodeImplementation, NodeType, SignalType,
};
use imp_registry::{get_node_type, Registry};

use crate::resolve_type_args::resolve_type_args;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationResolveError {
    pub node_id: Option<String>,
    pub message: String,
}

pub fn effective_node_implementation(node_type: &NodeType) -> NodeImplementation {
    node_type
        .implementation
        .clone()
        .unwrap_or_else(|| NodeImplementation::default_for(&node_type.id))
}

pub fn signal_type_matches(pattern: &SignalType, signal_type: &SignalType) -> bool {
    let SignalType::Concrete {
        id: pattern_id,
        args: pattern_args,
    } = pattern
    else {
        return false;
    };

    let SignalType::Concrete {
        id: type_id,
        args: type_args,
    } = signal_type
    else {
        return false;
    };

    if pattern_id == "any" {
        return true;
    }

    if pattern_id != type_id || pattern_args.len() != type_args.len() {
        return false;
    }

    pattern_args
        .iter()
        .zip(type_args.iter())
        .all(|(pattern_arg, type_arg)| signal_type_matches(pattern_arg, type_arg))
}

pub fn resolve_implementation(
    graph: &Graph,
    node_id: &str,
    registry: &Registry,
) -> Result<ImplementationId, ImplementationResolveError> {
    let node = graph.nodes.get(node_id).ok_or_else(|| ImplementationResolveError {
        node_id: None,
        message: format!("unknown node \"{node_id}\""),
    })?;

    let node_type = get_node_type(registry, &node.node_type).ok_or_else(|| {
        ImplementationResolveError {
            node_id: Some(node.id.clone()),
            message: format!("unknown NodeType \"{}\"", node.node_type),
        }
    })?;

    match effective_node_implementation(node_type) {
        NodeImplementation::Universal { id } => Ok(id),
        NodeImplementation::Dispatch {
            param_index,
            cases,
            default,
        } => {
            let type_args = resolve_type_args(graph, &node.id, registry).ok_or_else(|| {
                ImplementationResolveError {
                    node_id: Some(node.id.clone()),
                    message: "cannot resolve type arguments for dispatched implementation".into(),
                }
            })?;

            let index = param_index.unwrap_or(0);
            let selected = type_args.get(index).ok_or_else(|| ImplementationResolveError {
                node_id: Some(node.id.clone()),
                message: format!(
                    "dispatch paramIndex {index} out of range ({} type args)",
                    type_args.len()
                ),
            })?;

            for case_entry in &cases {
                if signal_type_matches(&case_entry.match_type, selected) {
                    return Ok(case_entry.implementation.clone());
                }
            }

            default.ok_or_else(|| ImplementationResolveError {
                node_id: Some(node.id.clone()),
                message: "no implementation case matches resolved type argument".into(),
            })
        }
    }
}
