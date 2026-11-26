//! Graph type checking.

use imp_core_types::{resolved_nodes, Edge, GraphSource, Node, NodeDefinition, Type};
use imp_registry::{get_node_definitions, Registry};

use crate::assignable::assignable;
use crate::check_type_definitions::type_reference_problems;
use crate::check_type_param_bounds::check_type_param_bounds;
use crate::normalize::with_prepared;
use crate::graph_analysis::{
    candidate_survives, catalog_port_type_from_definition, effective_definition, resolve_port_type,
    resolve_selections, PortDirection,
};
use crate::substitution::{
    apply_substitution, contains_type_var, scope_type, unscoped_param, Substitution,
    TypeCheckError,
};
use crate::unify::format_type;

/// Type-check a graph source. Pass `&graph` for a single serialized document.
pub fn check_graph<'a>(graph: impl Into<GraphSource<'a>>, registry: &Registry) -> Vec<TypeCheckError> {
    with_prepared(graph.into(), registry, check_graph_prepared)
}

fn check_graph_prepared(source: GraphSource<'_>, registry: &Registry) -> Vec<TypeCheckError> {
    let nodes = resolved_nodes(source);
    let mut errors = Vec::new();
    let resolved = resolve_selections(source, registry);
    let selected = resolved.selected;
    let mut edge_subst = resolved.subst;
    let unknown_types = resolved.unknown_types;

    for node_id in &unknown_types {
        let Some(node) = source.nodes.get_node(node_id) else {
            continue;
        };
        errors.push(TypeCheckError::at_node(
            node.id.clone(),
            format!("unknown NodeDefinition \"{}\"", node.node_type),
        ));
    }

    for node in &nodes {
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
                        source,
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
        let mut seen = std::collections::BTreeSet::new();
        for problem in node
            .type_args
            .iter()
            .flat_map(|arg| type_reference_problems(arg, registry))
        {
            if seen.insert(problem.clone()) {
                errors.push(TypeCheckError::at_node(node.id.clone(), problem));
            }
        }
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
            let inbound: Vec<_> = source
                .edges
                .edge_entries()
                .into_iter()
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

    for (_, edge) in source.edges.edge_entries() {
        let Some(from_node) = source.nodes.get_node(&edge.from.node) else {
            errors.push(TypeCheckError::at_node(
                edge.from.node.clone(),
                format!("unknown node \"{}\"", edge.from.node),
            ));
            continue;
        };
        let Some(to_node) = source.nodes.get_node(&edge.to.node) else {
            errors.push(TypeCheckError::at_node(
                edge.to.node.clone(),
                format!("unknown node \"{}\"", edge.to.node),
            ));
            continue;
        };

        let Some(from_def) = selected.get(&from_node.id) else {
            continue;
        };
        let Some(to_def) = selected.get(&to_node.id) else {
            continue;
        };

        let Some(output_type) = catalog_port_type_from_definition(
            &from_node,
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
            &to_node,
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
            Err(err) => {
                let message = type_param_conflict(
                    source,
                    &to_node,
                    to_def,
                    edge,
                    &output_type,
                    &input_type,
                    &edge_subst,
                )
                .unwrap_or(err.message);
                errors.push(TypeCheckError::at_port(
                    edge.to.node.clone(),
                    edge.to.port.clone(),
                    message,
                ))
            }
        }
    }

    for node in &nodes {
        let Some(definition) = selected.get(&node.id) else {
            continue;
        };

        for port_id in definition.outputs.keys() {
            if let Some(resolved) = resolve_port_type(
                source,
                &node.id,
                port_id,
                registry,
                &edge_subst,
                Some(&selected),
            ) {
                if contains_type_var(&resolved) {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!(
                            "unresolved type parameter on output: {}",
                            format_type(&resolved)
                        ),
                    ));
                }
            }
        }

        for port_id in definition.inputs.keys() {
            let has_edge = source
                .edges
                .edge_entries()
                .into_iter()
                .any(|(_, edge)| edge.to.node == node.id && edge.to.port == *port_id);
            if !has_edge {
                continue;
            }
            if let Some(resolved) = resolve_port_type(
                source,
                &node.id,
                port_id,
                registry,
                &edge_subst,
                Some(&selected),
            ) {
                if contains_type_var(&resolved) {
                    errors.push(TypeCheckError::at_port(
                        node.id.clone(),
                        port_id.clone(),
                        format!(
                            "unresolved type parameter on input: {}",
                            format_type(&resolved)
                        ),
                    ));
                }
            }
        }
    }

    errors.extend(check_type_param_bounds(source, registry, Some(&selected)));

    errors
}

/// Describes a failed edge whose target port is a type parameter already bound through another
/// input of the same node: names the parameter, both types and both ports.
fn type_param_conflict(
    source: GraphSource<'_>,
    to_node: &Node,
    to_def: &NodeDefinition,
    edge: &Edge,
    output_type: &Type,
    input_type: &Type,
    subst: &Substitution,
) -> Option<String> {
    let Type::Var { param } = input_type else {
        return None;
    };
    let bound = apply_substitution(input_type, subst);
    if contains_type_var(&bound) {
        return None;
    }
    let incoming = apply_substitution(output_type, subst);
    let effective = effective_definition(to_node, to_def)?;
    let other_port = source
        .edges
        .edge_entries()
        .into_iter()
        .filter(|(_, other)| {
            other.to.node == to_node.id
                && (other.to.port != edge.to.port || other.from != edge.from)
        })
        .find_map(|(_, other)| {
            let port = effective.inputs.get(&other.to.port)?;
            let scoped = scope_type(&port.ty, &to_node.id);
            (scoped == *input_type).then(|| other.to.port.clone())
        });
    let origin = match other_port {
        Some(port) => format!(" (from input {port})"),
        None => String::new(),
    };
    Some(format!(
        "type parameter {} is {}{origin} but input {} receives {}",
        unscoped_param(param),
        format_type(&bound),
        edge.to.port,
        format_type(&incoming),
    ))
}
