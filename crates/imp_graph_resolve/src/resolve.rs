//! flattenGraph and buildExecutionProgram implementations.

use imp_core_types::{
    is_graph_backed_definition, Edge, Graph, NodeDefinition, NodeId, PortReference, SignalType,
};
use imp_registry::{get_node_definitions, list_node_definitions, Registry};
use imp_typecheck::instantiate_node_definition;
use std::collections::BTreeMap;

pub type SubgraphKey = String;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionProgram {
    pub root: Graph,
    pub subgraphs: BTreeMap<SubgraphKey, Graph>,
}

pub fn subgraph_key(definition_id: &str, type_args: Option<&[SignalType]>) -> SubgraphKey {
    let args = type_args.unwrap_or(&[]);
    format!(
        "{definition_id}::{}",
        serde_json::to_string(args).unwrap_or_else(|_| "[]".into())
    )
}

fn prefix_id(instance_id: &NodeId, inner_id: &NodeId) -> NodeId {
    format!("{instance_id}::{inner_id}")
}

fn graph_backed_definition<'a>(
    registry: &'a Registry,
    type_id: &str,
) -> Option<&'a NodeDefinition> {
    get_node_definitions(registry, type_id)
        .iter()
        .find(|def| is_graph_backed_definition(def))
}

fn is_composite_node(node_type: &str, registry: &Registry) -> bool {
    graph_backed_definition(registry, node_type).is_some()
}

fn find_composite_nodes(graph: &Graph, registry: &Registry) -> Vec<NodeId> {
    graph
        .nodes
        .values()
        .filter(|node| is_composite_node(&node.node_type, registry))
        .map(|node| node.id.clone())
        .collect()
}

fn resolve_definition(node: &imp_core_types::Node, registry: &Registry) -> Result<NodeDefinition, String> {
    let def = graph_backed_definition(registry, &node.node_type).ok_or_else(|| {
        format!(
            "Node \"{}\" type \"{}\" is not a graph-backed definition",
            node.id, node.node_type
        )
    })?;
    if def.body.is_none() || def.bindings.is_none() {
        return Err(format!(
            "Node \"{}\" type \"{}\" is not a graph-backed definition",
            node.id, node.node_type
        ));
    }
    if node.type_args.is_empty() {
        return Ok(def.clone());
    }
    instantiate_node_definition(def, &node.type_args)
}

fn edges_from_boundary_output<'a>(body: &'a Graph, boundary_node_id: &NodeId) -> Vec<&'a Edge> {
    body.edges
        .values()
        .filter(|edge| edge.from.node == *boundary_node_id && edge.from.port == "value")
        .collect()
}

fn edges_to_boundary_input<'a>(body: &'a Graph, boundary_node_id: &NodeId) -> Vec<&'a Edge> {
    body.edges
        .values()
        .filter(|edge| edge.to.node == *boundary_node_id)
        .collect()
}

fn expand_composite_instance(
    graph: &Graph,
    instance_id: &NodeId,
    registry: &Registry,
) -> Result<Graph, String> {
    let instance = graph
        .nodes
        .get(instance_id)
        .ok_or_else(|| format!("Unknown composite instance \"{instance_id}\""))?;

    let definition = resolve_definition(instance, registry)?;
    let body = definition.body.as_ref().unwrap();
    let bindings = definition.bindings.as_ref().unwrap();

    let mut nodes = graph.nodes.clone();
    nodes.remove(instance_id);

    for (id, node) in &body.nodes {
        let prefixed_id = prefix_id(instance_id, id);
        nodes.insert(
            prefixed_id.clone(),
            imp_core_types::Node {
                id: prefixed_id,
                node_type: node.node_type.clone(),
                type_args: node.type_args.clone(),
                inputs: node.inputs.clone(),
            },
        );
    }

    let mut edges: BTreeMap<String, Edge> = BTreeMap::new();
    let mut edge_counter = 0usize;

    for edge in graph.edges.values() {
        if edge.from.node == *instance_id || edge.to.node == *instance_id {
            continue;
        }
        push_edge(
            &mut edges,
            &mut edge_counter,
            instance_id,
            edge.from.clone(),
            edge.to.clone(),
            edge.order,
        );
    }

    for edge in body.edges.values() {
        push_edge(
            &mut edges,
            &mut edge_counter,
            instance_id,
            PortReference {
                node: prefix_id(instance_id, &edge.from.node),
                port: edge.from.port.clone(),
            },
            PortReference {
                node: prefix_id(instance_id, &edge.to.node),
                port: edge.to.port.clone(),
            },
            edge.order,
        );
    }

    let mut parent_inbound: BTreeMap<String, Vec<&Edge>> = BTreeMap::new();
    for edge in graph.edges.values() {
        if edge.to.node != *instance_id {
            continue;
        }
        parent_inbound
            .entry(edge.to.port.clone())
            .or_default()
            .push(edge);
    }

    for (input_port, inbound) in parent_inbound {
        let boundary_id = bindings.inputs.get(&input_port).ok_or_else(|| {
            format!("Composite \"{instance_id}\" missing input binding for port \"{input_port}\"")
        })?;
        let outbound = edges_from_boundary_output(body, boundary_id);
        if outbound.is_empty() {
            return Err(format!(
                "Bound input \"{boundary_id}\" in definition \"{}\" has no outbound edges",
                definition.id
            ));
        }
        let prefixed_boundary = prefix_id(instance_id, boundary_id);
        nodes.remove(&prefixed_boundary);
        let stale: Vec<String> = edges
            .iter()
            .filter(|(_, existing)| existing.from.node == prefixed_boundary)
            .map(|(id, _)| id.clone())
            .collect();
        for id in stale {
            edges.remove(&id);
        }

        let declared = definition.inputs.get(&input_port);
        if declared.map(|p| p.variadic).unwrap_or(false) {
            let pack_id = prefix_id(instance_id, &format!("__pack_{input_port}"));
            let element_type = declared.unwrap().signal_type.clone();
            nodes.insert(
                pack_id.clone(),
                imp_core_types::Node {
                    id: pack_id.clone(),
                    node_type: "pack".into(),
                    type_args: vec![element_type],
                    inputs: BTreeMap::new(),
                },
            );
            let mut sorted = inbound;
            sorted.sort_by_key(|e| e.order.unwrap_or(0));
            for edge in sorted {
                push_edge(
                    &mut edges,
                    &mut edge_counter,
                    instance_id,
                    edge.from.clone(),
                    PortReference {
                        node: pack_id.clone(),
                        port: "items".into(),
                    },
                    edge.order,
                );
            }
            for body_edge in outbound {
                push_edge(
                    &mut edges,
                    &mut edge_counter,
                    instance_id,
                    PortReference {
                        node: pack_id.clone(),
                        port: "list".into(),
                    },
                    PortReference {
                        node: prefix_id(instance_id, &body_edge.to.node),
                        port: body_edge.to.port.clone(),
                    },
                    None,
                );
            }
        } else {
            if inbound.len() != 1 {
                return Err(format!(
                    "Composite \"{instance_id}\" non-variadic input \"{input_port}\" expected one edge, got {}",
                    inbound.len()
                ));
            }
            let edge = inbound[0];
            for body_edge in outbound {
                push_edge(
                    &mut edges,
                    &mut edge_counter,
                    instance_id,
                    edge.from.clone(),
                    PortReference {
                        node: prefix_id(instance_id, &body_edge.to.node),
                        port: body_edge.to.port.clone(),
                    },
                    None,
                );
            }
        }
    }

    for edge in graph.edges.values() {
        if edge.from.node != *instance_id {
            continue;
        }
        let output_port = &edge.from.port;
        let boundary_id = bindings.outputs.get(output_port).ok_or_else(|| {
            format!("Composite \"{instance_id}\" missing output binding for port \"{output_port}\"")
        })?;
        let inbound = edges_to_boundary_input(body, boundary_id);
        let value_inbound = inbound
            .iter()
            .find(|e| e.to.port == "value")
            .ok_or_else(|| {
                format!(
                    "Bound output \"{boundary_id}\" in definition \"{}\" has no inbound value edge",
                    definition.id
                )
            })?;
        push_edge(
            &mut edges,
            &mut edge_counter,
            instance_id,
            PortReference {
                node: prefix_id(instance_id, &value_inbound.from.node),
                port: value_inbound.from.port.clone(),
            },
            edge.to.clone(),
            edge.order,
        );
    }

    Ok(Graph { nodes, edges })
}

fn push_edge(
    edges: &mut BTreeMap<String, Edge>,
    edge_counter: &mut usize,
    instance_id: &NodeId,
    from: PortReference,
    to: PortReference,
    order: Option<i64>,
) {
    let id = format!("{instance_id}::edge{edge_counter}");
    *edge_counter += 1;
    edges.insert(id, Edge { from, to, order });
}

pub fn flatten_graph(graph: &Graph, registry: &Registry) -> Result<Graph, String> {
    let mut current = graph.clone();
    let mut visiting = std::collections::BTreeSet::new();

    loop {
        let composites = find_composite_nodes(&current, registry);
        if composites.is_empty() {
            return Ok(current);
        }

        let instance_id = composites[0].clone();
        if visiting.contains(&instance_id) {
            return Err(format!(
                "Composite instance cycle detected at \"{instance_id}\""
            ));
        }
        visiting.insert(instance_id.clone());
        current = expand_composite_instance(&current, &instance_id, registry)?;
    }
}

pub fn build_execution_program(
    graph: &Graph,
    registry: &Registry,
) -> Result<ExecutionProgram, String> {
    let mut subgraphs = BTreeMap::new();

    for node in graph.nodes.values() {
        let Some(def) = graph_backed_definition(registry, &node.node_type) else {
            continue;
        };

        let key = subgraph_key(&def.id, Some(&node.type_args));
        if subgraphs.contains_key(&key) {
            continue;
        }

        let resolved = if node.type_args.is_empty() {
            def.clone()
        } else {
            instantiate_node_definition(def, &node.type_args)?
        };
        let body = resolved
            .body
            .ok_or_else(|| format!("Graph-backed definition \"{}\" is missing body", def.id))?;
        subgraphs.insert(key, body);
    }

    for definition in list_node_definitions(registry) {
        if !is_graph_backed_definition(definition) {
            continue;
        }
        let Some(body) = &definition.body else {
            continue;
        };
        let default_key = subgraph_key(&definition.id, None);
        subgraphs.entry(default_key).or_insert_with(|| body.clone());
    }

    Ok(ExecutionProgram {
        root: graph.clone(),
        subgraphs,
    })
}

pub fn is_graph_backed_node_type(type_id: &str, registry: &Registry) -> bool {
    graph_backed_definition(registry, type_id).is_some()
}
