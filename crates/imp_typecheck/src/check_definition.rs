//! Graph-backed definition validation.
//! Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

use imp_core_types::{is_graph_backed_definition, Graph, NodeDefinition, SignalType};
use imp_registry::{get_node_definitions, list_node_definitions, Registry};

use crate::check::check_graph;
use crate::graph_analysis::{catalog_port_type, resolve_port_type, PortDirection};
use crate::substitution::{scope_signal_type, Substitution, TypeCheckError};
use crate::unify::unify;

fn scope_definition_port(signal_type: &SignalType, definition_id: &str) -> SignalType {
    scope_signal_type(signal_type, &format!("__def__{definition_id}"))
}

pub fn check_definition(
    definition: &NodeDefinition,
    registry: &Registry,
) -> Vec<TypeCheckError> {
    let mut errors = Vec::new();

    if !is_graph_backed_definition(definition) {
        if definition.bindings.is_some() {
            errors.push(TypeCheckError::for_definition(
                definition.id.clone(),
                "bindings are only valid on graph-backed definitions",
            ));
        }
        return errors;
    }

    if definition.implementation.is_some() {
        errors.push(TypeCheckError::for_definition(
            definition.id.clone(),
            "graph-backed definitions must not declare implementation",
        ));
    }

    let Some(body) = &definition.body else {
        errors.push(TypeCheckError::for_definition(
            definition.id.clone(),
            "graph-backed definitions require body and bindings",
        ));
        return errors;
    };
    let Some(bindings) = &definition.bindings else {
        errors.push(TypeCheckError::for_definition(
            definition.id.clone(),
            "graph-backed definitions require body and bindings",
        ));
        return errors;
    };

    for port_id in definition.inputs.keys() {
        if !bindings.inputs.contains_key(port_id) {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("missing input binding for port \"{port_id}\""),
                );
                err.port_id = Some(port_id.clone());
                err
            });
        }
    }

    for port_id in definition.outputs.keys() {
        if !bindings.outputs.contains_key(port_id) {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("missing output binding for port \"{port_id}\""),
                );
                err.port_id = Some(port_id.clone());
                err
            });
        }
    }

    let bound_input_nodes: std::collections::BTreeSet<_> =
        bindings.inputs.values().cloned().collect();
    let bound_output_nodes: std::collections::BTreeSet<_> =
        bindings.outputs.values().cloned().collect();

    for node in body.nodes.values() {
        if node.node_type == "input" && !bound_input_nodes.contains(&node.id) {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("unbound boundary input node \"{}\"", node.id),
                );
                err.node_id = Some(node.id.clone());
                err
            });
        }
        if node.node_type == "output" && !bound_output_nodes.contains(&node.id) {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("unbound boundary output node \"{}\"", node.id),
                );
                err.node_id = Some(node.id.clone());
                err
            });
        }
    }

    let subst = Substitution::new();

    for port_id in definition.inputs.keys() {
        let Some(boundary_id) = bindings.inputs.get(port_id) else {
            continue;
        };
        let Some(boundary) = body.nodes.get(boundary_id) else {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("bound input node \"{boundary_id}\" not found in body"),
                );
                err.port_id = Some(port_id.clone());
                err
            });
            continue;
        };
        if boundary.node_type != "input" {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("bound input node \"{boundary_id}\" must have type \"input\""),
                );
                err.node_id = Some(boundary_id.clone());
                err
            });
            continue;
        }
        let Some(boundary_type) =
            resolve_port_type(body, boundary_id, &"value".into(), registry, &subst, None)
        else {
            continue;
        };
        let Some(expected_port) = definition.inputs.get(port_id) else {
            continue;
        };
        let expected = if expected_port.variadic {
            imp_core_types::list_of(scope_definition_port(
                &expected_port.signal_type,
                &definition.id,
            ))
        } else {
            scope_definition_port(&expected_port.signal_type, &definition.id)
        };
        if let Err(err) = unify(&boundary_type, &expected, &subst) {
            errors.push({
                let mut tc_err = TypeCheckError::for_definition(definition.id.clone(), err.message);
                tc_err.node_id = Some(boundary_id.clone());
                tc_err.port_id = Some("value".into());
                tc_err
            });
        }
    }

    for port_id in definition.outputs.keys() {
        let Some(boundary_id) = bindings.outputs.get(port_id) else {
            continue;
        };
        let Some(boundary) = body.nodes.get(boundary_id) else {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("bound output node \"{boundary_id}\" not found in body"),
                );
                err.port_id = Some(port_id.clone());
                err
            });
            continue;
        };
        if boundary.node_type != "output" {
            errors.push({
                let mut err = TypeCheckError::for_definition(
                    definition.id.clone(),
                    format!("bound output node \"{boundary_id}\" must have type \"output\""),
                );
                err.node_id = Some(boundary_id.clone());
                err
            });
            continue;
        }
        let Some(boundary_type) =
            catalog_port_type(boundary, registry, &"value".into(), PortDirection::Input, None)
        else {
            continue;
        };
        let Some(expected_port) = definition.outputs.get(port_id) else {
            continue;
        };
        let expected = scope_definition_port(&expected_port.signal_type, &definition.id);
        if let Err(err) = unify(&boundary_type, &expected, &subst) {
            errors.push({
                let mut tc_err = TypeCheckError::for_definition(definition.id.clone(), err.message);
                tc_err.node_id = Some(boundary_id.clone());
                tc_err.port_id = Some("value".into());
                tc_err
            });
        }
    }

    for mut err in check_graph(body, registry) {
        err.definition_id = Some(definition.id.clone());
        errors.push(err);
    }

    errors
}

fn collect_definition_refs(body: &Graph, registry: &Registry) -> Vec<String> {
    let mut refs = Vec::new();
    for node in body.nodes.values() {
        let overloads = get_node_definitions(registry, &node.node_type);
        if overloads.iter().any(is_graph_backed_definition) {
            refs.push(node.node_type.clone());
        }
    }
    refs
}

fn detect_definition_cycle(
    start_id: &str,
    registry: &Registry,
    visiting: &mut std::collections::BTreeSet<String>,
    visited: &mut std::collections::BTreeSet<String>,
) -> bool {
    if visiting.contains(start_id) {
        return true;
    }
    if visited.contains(start_id) {
        return false;
    }

    visiting.insert(start_id.to_string());
    for def in get_node_definitions(registry, start_id) {
        if let Some(body) = &def.body {
            for reference in collect_definition_refs(body, registry) {
                if detect_definition_cycle(&reference, registry, visiting, visited) {
                    return true;
                }
            }
        }
    }
    visiting.remove(start_id);
    visited.insert(start_id.to_string());
    false
}

pub fn check_graph_definitions(registry: &Registry) -> Vec<TypeCheckError> {
    let mut errors = Vec::new();

    for definition in list_node_definitions(registry) {
        if !is_graph_backed_definition(definition) {
            continue;
        }

        errors.extend(check_definition(definition, registry));

        let mut visiting = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        if detect_definition_cycle(&definition.id, registry, &mut visiting, &mut visited) {
            errors.push(TypeCheckError::for_definition(
                definition.id.clone(),
                format!("definition-reference cycle involving \"{}\"", definition.id),
            ));
        }
    }

    errors
}
