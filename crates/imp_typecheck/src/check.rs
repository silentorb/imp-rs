//! Graph type checking.

use imp_core_types::{Graph, GraphType, GraphTypeId, Node, NodeId, NodeType, PortId, SignalType};
use imp_registry::{get_graph_type, get_node_type, Registry};

use crate::check_type_param_bounds::check_type_param_bounds;
use crate::instantiate::instantiate_node_type;
use crate::substitution::{
    apply_substitution, contains_type_var, scope_signal_type, Substitution, TypeCheckError,
};
use crate::unify::{format_signal_type, unify};

fn effective_node_type(node: &Node, registry: &Registry) -> Option<NodeType> {
    let catalog = get_node_type(registry, &node.node_type)?;
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
        let Some(node_type) = get_node_type(registry, &node.node_type) else {
            errors.push(TypeCheckError {
                node_id: Some(node.id.clone()),
                port_id: None,
                message: format!("unknown NodeType \"{}\"", node.node_type),
            });
            continue;
        };

        if !node.type_args.is_empty() {
            let expected = node_type.type_params.len();
            if node.type_args.len() != expected {
                errors.push(TypeCheckError {
                    node_id: Some(node.id.clone()),
                    port_id: None,
                    message: format!("expected {expected} typeArgs, got {}", node.type_args.len()),
                });
            }
        }

        let effective = effective_node_type(node, registry).unwrap_or_else(|| node_type.clone());

        for (port_id, port) in &effective.inputs {
            let has_edge = graph.edges.values().any(|edge| {
                edge.to.node == node.id && edge.to.port == *port_id
            });
            let has_literal = node.inputs.contains_key(port_id);
            let has_default = port.default_value.is_some();
            if !has_edge && !has_literal && !has_default {
                errors.push(TypeCheckError {
                    node_id: Some(node.id.clone()),
                    port_id: Some(port_id.clone()),
                    message: format!("required input port \"{port_id}\" is unsatisfied"),
                });
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
            errors.push(TypeCheckError {
                node_id: Some(edge.from.node.clone()),
                port_id: Some(edge.from.port.clone()),
                message: "unknown output port".into(),
            });
            continue;
        };
        let Some(input_type) =
            catalog_port_type(to_node, registry, &edge.to.port, PortDirection::Input)
        else {
            errors.push(TypeCheckError {
                node_id: Some(edge.to.node.clone()),
                port_id: Some(edge.to.port.clone()),
                message: "unknown input port".into(),
            });
            continue;
        };

        match unify(&output_type, &input_type, &subst) {
            Ok(next) => subst = next,
            Err(err) => errors.push(TypeCheckError {
                node_id: Some(edge.to.node.clone()),
                port_id: Some(edge.to.port.clone()),
                message: err.message,
            }),
        }
    }

    for node in graph.nodes.values() {
        let Some(node_type) = effective_node_type(node, registry)
            .or_else(|| get_node_type(registry, &node.node_type).cloned())
        else {
            continue;
        };

        for port_id in node_type.outputs.keys() {
            if let Some(resolved) = resolve_port_type(graph, &node.id, port_id, registry, &subst) {
                if contains_type_var(&resolved) {
                    errors.push(TypeCheckError {
                        node_id: Some(node.id.clone()),
                        port_id: Some(port_id.clone()),
                        message: format!(
                            "unresolved type parameter on output: {}",
                            format_signal_type(&resolved)
                        ),
                    });
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
                    errors.push(TypeCheckError {
                        node_id: Some(node.id.clone()),
                        port_id: Some(port_id.clone()),
                        message: format!(
                            "unresolved type parameter on input: {}",
                            format_signal_type(&resolved)
                        ),
                    });
                }
            }
        }
    }

    errors.extend(check_type_param_bounds(graph, registry));

    errors
}

pub fn check_graph_implements(
    graph: &Graph,
    graph_type_id: &GraphTypeId,
    registry: &Registry,
) -> Vec<TypeCheckError> {
    let Some(graph_type) = get_graph_type(registry, graph_type_id) else {
        return vec![TypeCheckError {
            node_id: None,
            port_id: None,
            message: format!("unknown GraphType \"{graph_type_id}\""),
        }];
    };

    let mut errors = Vec::new();
    let subst = Substitution::new();

    let input_nodes: Vec<&Node> = graph
        .nodes
        .values()
        .filter(|node| node.node_type == "input")
        .collect();
    let output_nodes: Vec<&Node> = graph
        .nodes
        .values()
        .filter(|node| node.node_type == "output")
        .collect();

    for port_id in graph_type.inputs.keys() {
        let port = &graph_type.inputs[port_id];
        let Some(boundary) = input_nodes.first() else {
            errors.push(TypeCheckError {
                node_id: None,
                port_id: Some(port_id.clone()),
                message: format!("missing boundary input for GraphType port \"{port_id}\""),
            });
            continue;
        };
        if let Some(boundary_type) =
            resolve_port_type(graph, &boundary.id, &"value".into(), registry, &subst)
        {
            let expected = scope_graph_type_port(&port.signal_type, graph_type);
            if let Err(err) = unify(&boundary_type, &expected, &subst) {
                errors.push(TypeCheckError {
                    node_id: Some(boundary.id.clone()),
                    port_id: Some("value".into()),
                    message: err.message,
                });
            }
        }
    }

    for port_id in graph_type.outputs.keys() {
        let port = &graph_type.outputs[port_id];
        let Some(boundary) = output_nodes.first() else {
            errors.push(TypeCheckError {
                node_id: None,
                port_id: Some(port_id.clone()),
                message: format!("missing boundary output for GraphType port \"{port_id}\""),
            });
            continue;
        };
        if let Some(boundary_type) =
            catalog_port_type(boundary, registry, &"value".into(), PortDirection::Input)
        {
            let expected = scope_graph_type_port(&port.signal_type, graph_type);
            if let Err(err) = unify(&boundary_type, &expected, &subst) {
                errors.push(TypeCheckError {
                    node_id: Some(boundary.id.clone()),
                    port_id: Some("value".into()),
                    message: err.message,
                });
            }
        }
    }

    errors
}

fn scope_graph_type_port(signal_type: &SignalType, graph_type: &GraphType) -> SignalType {
    let scope_id = format!("__graph__{}", graph_type.id);
    scope_signal_type(signal_type, &scope_id)
}
