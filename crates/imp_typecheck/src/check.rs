//! Graph type checking.

use imp_core_types::{Graph, Node, NodeId, NodeDefinition, PortId, SignalType};
use imp_registry::{get_node_definition, Registry};

use crate::check_type_param_bounds::check_type_param_bounds;
use crate::instantiate::instantiate_node_type;
use crate::substitution::{
    apply_substitution, contains_type_var, scope_signal_type, Substitution, TypeCheckError,
};
use crate::unify::{format_signal_type, unify};

fn effective_node_type(node: &Node, registry: &Registry) -> Option<NodeDefinition> {
    let catalog = get_node_definition(registry, &node.node_type)?;
    if node.type_args.is_empty() {
        return Some(catalog.clone());
    }
    instantiate_node_type(catalog, &node.type_args).ok()
}

fn catalog_port_type(
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

enum PortDirection {
    Input,
    Output,
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

    for edge in graph.edges.values() {
        if edge.to.node == *node_id && edge.to.port == *port_id {
            if let Some(from_node) = graph.nodes.get(&edge.from.node) {
                if let Some(source_type) = catalog_port_type(
                    from_node,
                    registry,
                    &edge.from.port,
                    PortDirection::Output,
                ) {
                    if let Ok(next) = unify(&source_type, &input_type, subst) {
                        input_type = apply_substitution(&input_type, &next);
                    }
                }
            }
        }
    }

    Some(apply_substitution(&input_type, subst))
}

pub fn check_graph(graph: &Graph, registry: &Registry) -> Vec<TypeCheckError> {
    let mut errors = Vec::new();
    let mut subst = Substitution::new();

    for node in graph.nodes.values() {
        let Some(node_type) = get_node_definition(registry, &node.node_type) else {
            errors.push(TypeCheckError::at_node(
                node.id.clone(),
                format!("unknown NodeDefinition \"{}\"", node.node_type),
            ));
            continue;
        };

        if !node.type_args.is_empty() {
            let expected = node_type.type_params.len();
            if node.type_args.len() != expected {
                errors.push(TypeCheckError::at_node(
                    node.id.clone(),
                    format!("expected {expected} typeArgs, got {}", node.type_args.len()),
                ));
            }
        }

        let effective = effective_node_type(node, registry).unwrap_or_else(|| node_type.clone());

        for (port_id, port) in &effective.inputs {
            let has_edge = graph
                .edges
                .values()
                .any(|edge| edge.to.node == node.id && edge.to.port == *port_id);
            let has_literal = node.inputs.contains_key(port_id);
            let has_default = port.default_value.is_some();
            if !has_edge && !has_literal && !has_default {
                errors.push(TypeCheckError::at_port(
                    node.id.clone(),
                    port_id.clone(),
                    format!("required input port \"{port_id}\" is unsatisfied"),
                ));
            }
        }
    }

    for edge in graph.edges.values() {
        let Some(from_node) = graph.nodes.get(&edge.from.node) else {
            continue;
        };
        let Some(to_node) = graph.nodes.get(&edge.to.node) else {
            continue;
        };

        let Some(output_type) =
            catalog_port_type(from_node, registry, &edge.from.port, PortDirection::Output)
        else {
            errors.push(TypeCheckError::at_port(
                edge.from.node.clone(),
                edge.from.port.clone(),
                "unknown output port",
            ));
            continue;
        };
        let Some(input_type) =
            catalog_port_type(to_node, registry, &edge.to.port, PortDirection::Input)
        else {
            errors.push(TypeCheckError::at_port(
                edge.to.node.clone(),
                edge.to.port.clone(),
                "unknown input port",
            ));
            continue;
        };

        match unify(&output_type, &input_type, &subst) {
            Ok(next) => subst = next,
            Err(err) => errors.push(TypeCheckError::at_port(
                edge.to.node.clone(),
                edge.to.port.clone(),
                err.message,
            )),
        }
    }

    for node in graph.nodes.values() {
        let Some(node_type) = effective_node_type(node, registry)
            .or_else(|| get_node_definition(registry, &node.node_type).cloned())
        else {
            continue;
        };

        for port_id in node_type.outputs.keys() {
            if let Some(resolved) = resolve_port_type(graph, &node.id, port_id, registry, &subst) {
                if contains_type_var(&resolved) {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!(
                            "unresolved type parameter on output: {}",
                            format_signal_type(&resolved)
                        ),
                    ));
                }
            }
        }

        for port_id in node_type.inputs.keys() {
            let has_edge = graph
                .edges
                .values()
                .any(|edge| edge.to.node == node.id && edge.to.port == *port_id);
            if !has_edge {
                continue;
            }
            if let Some(resolved) = resolve_port_type(graph, &node.id, port_id, registry, &subst) {
                if contains_type_var(&resolved) {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!(
                            "unresolved type parameter on input: {}",
                            format_signal_type(&resolved)
                        ),
                    ));
                }
            }
        }
    }

    errors.extend(check_type_param_bounds(graph, registry));

    errors
}
