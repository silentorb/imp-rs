//! Implementation resolution from type arguments.

use imp_core_types::{
    is_graph_backed_definition, GraphSource, ImplementationId, NodeDefinition, NodeImplementation,
    SignalType,
};
use imp_registry::Registry;

use crate::graph_analysis::resolve_selections;
use crate::resolve_type_args::resolve_type_args;
use crate::subtype::is_subtype_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationResolveError {
    pub node_id: Option<String>,
    pub message: String,
}

pub fn effective_node_implementation(definition: &NodeDefinition) -> NodeImplementation {
    definition
        .implementation
        .clone()
        .unwrap_or_else(|| NodeImplementation::default_for(&definition.id))
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

    if pattern_id != type_id && !is_subtype_id(type_id, pattern_id) {
        return false;
    }

    if pattern_args.len() != type_args.len() {
        return false;
    }

    pattern_args
        .iter()
        .zip(type_args.iter())
        .all(|(pattern_arg, type_arg)| signal_type_matches(pattern_arg, type_arg))
}

pub fn resolve_implementation<'a>(
    graph: impl Into<GraphSource<'a>>,
    node_id: &str,
    registry: &Registry,
) -> Result<ImplementationId, ImplementationResolveError> {
    let source: GraphSource<'a> = graph.into();
    let node = source.nodes.get_node(&node_id.to_string()).ok_or_else(|| ImplementationResolveError {
        node_id: None,
        message: format!("unknown node \"{node_id}\""),
    })?;

    let selected = resolve_selections(source, registry).selected;
    let definition = selected.get(&node.id).ok_or_else(|| ImplementationResolveError {
        node_id: Some(node.id.clone()),
        message: format!(
            "unknown or unresolved NodeDefinition \"{}\"",
            node.node_type
        ),
    })?;

    if is_graph_backed_definition(definition) {
        return Err(ImplementationResolveError {
            node_id: Some(node.id.clone()),
            message: format!(
                "graph-backed definition \"{}\" has no implementation id",
                node.node_type
            ),
        });
    }

    match effective_node_implementation(definition) {
        NodeImplementation::Universal { id } => Ok(id),
        NodeImplementation::Dispatch {
            param_index,
            cases,
            default,
        } => {
            let type_args =
                resolve_type_args(source, &node.id, registry, Some(&selected)).ok_or_else(|| {
                    ImplementationResolveError {
                        node_id: Some(node.id.clone()),
                        message: "cannot resolve type arguments for dispatched implementation"
                            .into(),
                    }
                })?;

            let index = param_index.unwrap_or(0);
            let selected_arg = type_args.get(index).ok_or_else(|| ImplementationResolveError {
                node_id: Some(node.id.clone()),
                message: format!(
                    "dispatch paramIndex {index} out of range ({} type args)",
                    type_args.len()
                ),
            })?;

            for case_entry in &cases {
                if signal_type_matches(&case_entry.match_type, selected_arg) {
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
