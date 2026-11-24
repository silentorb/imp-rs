//! Shared graph analysis for type checking and implementation dispatch.

use imp_core_types::{resolved_nodes, GraphSource, Node, NodeDefinition, NodeId, PortId, SignalType};
use imp_registry::{get_node_definitions, Registry};
use std::collections::BTreeMap;

use crate::assignable::assignable;
use crate::instantiate::instantiate_node_type;
use crate::substitution::{apply_substitution, scope_signal_type, Substitution};

pub type SelectedDefinitions = BTreeMap<NodeId, NodeDefinition>;

#[derive(Debug, Clone, Copy)]
pub enum PortDirection {
    Input,
    Output,
}

pub fn effective_definition(node: &Node, definition: &NodeDefinition) -> Option<NodeDefinition> {
    if node.type_args.is_empty() {
        return Some(definition.clone());
    }
    instantiate_node_type(definition, &node.type_args).ok()
}

/// Prefer selected overload + [`effective_definition`].
pub fn effective_node_type(node: &Node, registry: &Registry) -> Option<NodeDefinition> {
    let overloads = get_node_definitions(registry, &node.node_type);
    if overloads.len() != 1 {
        return None;
    }
    effective_definition(node, &overloads[0])
}

pub fn catalog_port_type_from_definition(
    node: &Node,
    definition: &NodeDefinition,
    port_id: &PortId,
    direction: PortDirection,
) -> Option<SignalType> {
    let effective = effective_definition(node, definition)?;
    let port = match direction {
        PortDirection::Input => effective.inputs.get(port_id)?,
        PortDirection::Output => effective.outputs.get(port_id)?,
    };
    Some(scope_signal_type(&port.signal_type, &node.id))
}

pub fn catalog_port_type(
    node: &Node,
    registry: &Registry,
    port_id: &PortId,
    direction: PortDirection,
    selected: Option<&SelectedDefinitions>,
) -> Option<SignalType> {
    let definition = selected
        .and_then(|map| map.get(&node.id).cloned())
        .or_else(|| effective_node_type(node, registry))?;
    catalog_port_type_from_definition(node, &definition, port_id, direction)
}

/// Try whether `candidate` is consistent with edges touching `node_id` whose
/// other endpoint already has a selected definition. Unresolved neighbors do not
/// constrain the candidate; `check_graph` reports them.
pub fn candidate_survives(
    source: GraphSource<'_>,
    node_id: &NodeId,
    candidate: &NodeDefinition,
    _registry: &Registry,
    selected: &SelectedDefinitions,
    subst: &Substitution,
) -> Option<Substitution> {
    let node = source.nodes.get_node(node_id)?;
    let mut trial = subst.clone();

    for (_, edge) in source.edges.edge_entries() {
        if edge.to.node == *node_id {
            let Some(from_node) = source.nodes.get_node(&edge.from.node) else {
                continue;
            };
            let Some(from_def) = selected.get(&from_node.id) else {
                continue;
            };
            let output_type = catalog_port_type_from_definition(
                &from_node,
                from_def,
                &edge.from.port,
                PortDirection::Output,
            )?;
            let input_type = catalog_port_type_from_definition(
                &node,
                candidate,
                &edge.to.port,
                PortDirection::Input,
            )?;
            trial = assignable(&output_type, &input_type, &trial).ok()?;
        }

        if edge.from.node == *node_id {
            let Some(to_node) = source.nodes.get_node(&edge.to.node) else {
                continue;
            };
            let Some(to_def) = selected.get(&to_node.id) else {
                continue;
            };
            let output_type = catalog_port_type_from_definition(
                &node,
                candidate,
                &edge.from.port,
                PortDirection::Output,
            )?;
            let input_type = catalog_port_type_from_definition(
                &to_node,
                to_def,
                &edge.to.port,
                PortDirection::Input,
            )?;
            trial = assignable(&output_type, &input_type, &trial).ok()?;
        }
    }

    Some(trial)
}

pub struct ResolveSelections {
    pub selected: SelectedDefinitions,
    pub subst: Substitution,
    pub unknown_types: Vec<NodeId>,
}

/// Fixed-point overload selection + substitution accumulation over the source's resolved members.
pub fn resolve_selections<'a>(
    graph: impl Into<GraphSource<'a>>,
    registry: &Registry,
) -> ResolveSelections {
    let source = graph.into();
    let nodes = resolved_nodes(source);
    let mut selected = SelectedDefinitions::new();
    let mut unknown_types = Vec::new();
    let mut subst = Substitution::new();

    for node in &nodes {
        let overloads = get_node_definitions(registry, &node.node_type);
        if overloads.is_empty() {
            unknown_types.push(node.id.clone());
            continue;
        }
        if overloads.len() == 1 {
            selected.insert(node.id.clone(), overloads[0].clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        for node in &nodes {
            if selected.contains_key(&node.id) {
                continue;
            }
            let overloads = get_node_definitions(registry, &node.node_type);
            if overloads.len() <= 1 {
                continue;
            }

            let mut survivors: Vec<(NodeDefinition, Substitution)> = Vec::new();
            for candidate in overloads {
                if let Some(trial) =
                    candidate_survives(source, &node.id, candidate, registry, &selected, &subst)
                {
                    survivors.push((candidate.clone(), trial));
                }
            }

            if survivors.len() == 1 {
                let (definition, next_subst) = survivors.remove(0);
                selected.insert(node.id.clone(), definition);
                subst = next_subst;
                changed = true;
            }
        }

        for (_, edge) in source.edges.edge_entries() {
            let Some(from_node) = source.nodes.get_node(&edge.from.node) else {
                continue;
            };
            let Some(to_node) = source.nodes.get_node(&edge.to.node) else {
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
                continue;
            };
            let Some(input_type) = catalog_port_type_from_definition(
                &to_node,
                to_def,
                &edge.to.port,
                PortDirection::Input,
            ) else {
                continue;
            };

            if let Ok(next) = assignable(&output_type, &input_type, &subst) {
                if next != subst {
                    subst = next;
                    changed = true;
                }
            }
        }
    }

    ResolveSelections {
        selected,
        subst,
        unknown_types,
    }
}

pub fn collect_graph_substitution<'a>(
    graph: impl Into<GraphSource<'a>>,
    registry: &Registry,
) -> Substitution {
    resolve_selections(graph, registry).subst
}

pub fn resolve_port_type<'a>(
    graph: impl Into<GraphSource<'a>>,
    node_id: &NodeId,
    port_id: &PortId,
    registry: &Registry,
    subst: &Substitution,
    selected: Option<&SelectedDefinitions>,
) -> Option<SignalType> {
    let source = graph.into();
    let node = source.nodes.get_node(node_id)?;
    let overloads = get_node_definitions(registry, &node.node_type);
    let definition = selected
        .and_then(|map| map.get(node_id).cloned())
        .or_else(|| {
            if overloads.len() == 1 {
                Some(overloads[0].clone())
            } else {
                None
            }
        })?;

    if let Some(output_type) =
        catalog_port_type_from_definition(&node, &definition, port_id, PortDirection::Output)
    {
        return Some(apply_substitution(&output_type, subst));
    }

    let effective = effective_definition(&node, &definition)?;
    let input_template = effective.inputs.get(port_id)?;
    let mut input_type = scope_signal_type(&input_template.signal_type, &node.id);
    let mut local_subst = subst.clone();

    for (_, edge) in source.edges.edge_entries() {
        if edge.to.node == *node_id && edge.to.port == *port_id {
            if let Some(from_node) = source.nodes.get_node(&edge.from.node) {
                if let Some(source_type) = catalog_port_type(
                    &from_node,
                    registry,
                    &edge.from.port,
                    PortDirection::Output,
                    selected,
                ) {
                    if let Ok(next) = assignable(&source_type, &input_type, &local_subst) {
                        local_subst = next;
                        input_type = apply_substitution(&input_type, &local_subst);
                    }
                }
            }
        }
    }

    Some(apply_substitution(&input_type, &local_subst))
}
