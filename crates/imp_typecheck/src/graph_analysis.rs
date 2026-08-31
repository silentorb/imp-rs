//! Shared graph analysis for type checking and implementation dispatch.

use imp_core_types::{Graph, Node, NodeId, PortId, SignalType};
use imp_registry::{get_node_type, Registry};

use crate::instantiate::instantiate_node_type;
use crate::substitution::{apply_substitution, scope_signal_type, Substitution};
use crate::unify::unify;

pub fn effective_node_type(node: &Node, registry: &Registry) -> Option<imp_core_types::NodeType> {
    let catalog = get_node_type(registry, &node.node_type)?;
    if node.type_args.is_empty() {
        return Some(catalog.clone());
    }
    instantiate_node_type(catalog, &node.type_args).ok()
}

pub fn catalog_port_type(
    node: &Node,
    registry: &Registry,
    port_id: &PortId,
    direction: PortDirection,
) -> Option<SignalType> {
    let node_type = effective_node_type(node, registry)?;
    let port = match direction {
        PortDirection::Input => node_type.inputs.get(port_id)?,
        PortDirection::Output => node_type.outputs.get(port_id)?,
    };
    Some(scope_signal_type(&port.signal_type, &node.id))
}

#[derive(Debug, Clone, Copy)]
pub enum PortDirection {
    Input,
    Output,
}

pub fn collect_graph_substitution(graph: &Graph, registry: &Registry) -> Substitution {
    let mut subst = Substitution::new();

    for edge in graph.edges.values() {
        let Some(from_node) = graph.nodes.get(&edge.from.node) else {
            continue;
        };
        let Some(to_node) = graph.nodes.get(&edge.to.node) else {
            continue;
        };

        let Some(output_type) = catalog_port_type(from_node, registry, &edge.from.port, PortDirection::Output) else {
            continue;
        };
        let Some(input_type) = catalog_port_type(to_node, registry, &edge.to.port, PortDirection::Input) else {
            continue;
        };

        if let Ok(next) = unify(&output_type, &input_type, &subst) {
            subst = next;
        }
    }

    subst
}

pub fn resolve_port_type(
    graph: &Graph,
    node_id: &NodeId,
    port_id: &PortId,
    registry: &Registry,
    subst: &Substitution,
) -> Option<SignalType> {
    let node = graph.nodes.get(node_id)?;
    let node_type = effective_node_type(node, registry)?;

    if let Some(output_type) = catalog_port_type(node, registry, port_id, PortDirection::Output) {
        return Some(apply_substitution(&output_type, subst));
    }

    let input_template = node_type.inputs.get(port_id)?;
    let mut input_type = scope_signal_type(&input_template.signal_type, &node.id);
    let mut local_subst = subst.clone();

    for edge in graph.edges.values() {
        if edge.to.node == *node_id && edge.to.port == *port_id {
            if let Some(from_node) = graph.nodes.get(&edge.from.node) {
                if let Some(source_type) =
                    catalog_port_type(from_node, registry, &edge.from.port, PortDirection::Output)
                {
                    if let Ok(next) = unify(&source_type, &input_type, &local_subst) {
                        local_subst = next;
                        input_type = apply_substitution(&input_type, &local_subst);
                    }
                }
            }
        }
    }

    Some(apply_substitution(&input_type, &local_subst))
}
