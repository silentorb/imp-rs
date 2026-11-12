//! Graph type checking.

use imp_core_types::Graph;
use imp_registry::{get_node_definitions, Registry};

use crate::assignable::assignable;
use crate::check_type_param_bounds::check_type_param_bounds;
use crate::graph_analysis::{
    candidate_survives, catalog_port_type_from_definition, effective_definition, resolve_port_type,
    resolve_selections, PortDirection,
};
use crate::substitution::{contains_type_var, TypeCheckError};
use crate::unify::format_signal_type;

pub fn check_graph(graph: &Graph, registry: &Registry) -> Vec<TypeCheckError> {
    let mut errors = Vec::new();
    let resolved = resolve_selections(graph, registry);
    let selected = resolved.selected;
    let mut edge_subst = resolved.subst;
    let unknown_types = resolved.unknown_types;

    for node_id in &unknown_types {
        let Some(node) = graph.nodes.get(node_id) else {
            continue;
        };
        errors.push(TypeCheckError::at_node(
            node.id.clone(),
            format!("unknown NodeDefinition \"{}\"", node.node_type),
        ));
    }

    for node in graph.nodes.values() {
        if unknown_types.contains(&node.id) {
            continue;
        }
        let overloads = get_node_definitions(registry, &node.node_type);
        if overloads.is_empty() {
            continue;
        }

        if !selected.contains_key(&node.id) {
            let survivors: Vec<_> = overloads
                .iter()
                .filter(|candidate| {
                    candidate_survives(
                        graph,
                        &node.id,
                        candidate,
                        registry,
                        &selected,
                        &edge_subst,
                    )
                    .is_some()
                })
                .collect();
            if survivors.is_empty() {
                errors.push(TypeCheckError::at_node(
                    node.id.clone(),
                    format!("no matching overload for \"{}\"", node.node_type),
                ));
            } else {
                errors.push(TypeCheckError::at_node(
                    node.id.clone(),
                    format!(
                        "ambiguous overload for \"{}\" ({} candidates)",
                        node.node_type,
                        survivors.len()
                    ),
                ));
            }
            continue;
        }

        let definition = selected.get(&node.id).unwrap();
        if !node.type_args.is_empty() {
            let expected = definition.type_params.len();
            if node.type_args.len() != expected {
                errors.push(TypeCheckError::at_node(
                    node.id.clone(),
                    format!("expected {expected} typeArgs, got {}", node.type_args.len()),
                ));
            }
        }

        let Some(effective) = effective_definition(node, definition) else {
            continue;
        };

        for (port_id, port) in &effective.inputs {
            let inbound: Vec<_> = graph
                .edges
                .iter()
                .filter(|(_, edge)| edge.to.node == node.id && edge.to.port == *port_id)
                .collect();
            let has_edge = !inbound.is_empty();
            let has_literal = node.inputs.contains_key(port_id);
            let has_default = port.default_value.is_some();
            if port.variadic {
                if !has_edge && !has_default {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!("required input port \"{port_id}\" is unsatisfied"),
                    ));
                }
                let mut orders = std::collections::BTreeMap::<i64, String>::new();
                for (edge_id, edge) in &inbound {
                    match edge.order {
                        None => errors.push(TypeCheckError::at_port(
                            node.id.clone(),
                            port_id.clone(),
                            format!(
                                "variadic input \"{port_id}\" requires Edge.order on edge \"{edge_id}\""
                            ),
                        )),
                        Some(order) => {
                            if let Some(prior) = orders.insert(order, (*edge_id).clone()) {
                                errors.push(TypeCheckError::at_port(
                                    node.id.clone(),
                                    port_id.clone(),
                                    format!(
                                        "duplicate Edge.order {order} on variadic input \"{port_id}\" (edges \"{prior}\" and \"{edge_id}\")"
                                    ),
                                ));
                            }
                        }
                    }
                }
            } else {
                if inbound.len() > 1 {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!("multiple edges target non-variadic input \"{port_id}\""),
                    ));
                }
                if !has_edge && !has_literal && !has_default {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!("required input port \"{port_id}\" is unsatisfied"),
                    ));
                }
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

        let Some(from_def) = selected.get(&from_node.id) else {
            continue;
        };
        let Some(to_def) = selected.get(&to_node.id) else {
            continue;
        };

        let Some(output_type) = catalog_port_type_from_definition(
            from_node,
            from_def,
            &edge.from.port,
            PortDirection::Output,
        ) else {
            errors.push(TypeCheckError::at_port(
                edge.from.node.clone(),
                edge.from.port.clone(),
                "unknown output port",
            ));
            continue;
        };
        let Some(input_type) = catalog_port_type_from_definition(
            to_node,
            to_def,
            &edge.to.port,
            PortDirection::Input,
        ) else {
            errors.push(TypeCheckError::at_port(
                edge.to.node.clone(),
                edge.to.port.clone(),
                "unknown input port",
            ));
            continue;
        };

        match assignable(&output_type, &input_type, &edge_subst) {
            Ok(next) => edge_subst = next,
            Err(err) => errors.push(TypeCheckError::at_port(
                edge.to.node.clone(),
                edge.to.port.clone(),
                err.message,
            )),
        }
    }

    for node in graph.nodes.values() {
        let Some(definition) = selected.get(&node.id) else {
            continue;
        };

        for port_id in definition.outputs.keys() {
            if let Some(resolved) =
                resolve_port_type(graph, &node.id, port_id, registry, &edge_subst, Some(&selected))
            {
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

        for port_id in definition.inputs.keys() {
            let has_edge = graph
                .edges
                .values()
                .any(|edge| edge.to.node == node.id && edge.to.port == *port_id);
            if !has_edge {
                continue;
            }
            if let Some(resolved) =
                resolve_port_type(graph, &node.id, port_id, registry, &edge_subst, Some(&selected))
            {
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

    errors.extend(check_type_param_bounds(graph, registry, Some(&selected)));

    errors
}
