//! NodeDefinition instantiation.

use imp_core_types::{Graph, NodeDefinition, Port, Ports, SignalType};

use crate::substitution::assert_concrete_args;

fn substitute_in_type(
    signal_type: &SignalType,
    params: &[String],
    args: &[SignalType],
) -> Result<SignalType, String> {
    match signal_type {
        SignalType::Var { param } => {
            let index = params
                .iter()
                .position(|candidate| candidate == param)
                .ok_or_else(|| format!("unknown type parameter \"{param}\""))?;
            Ok(args[index].clone())
        }
        SignalType::Concrete { id, args: type_args } if !type_args.is_empty() => Ok(
            SignalType::Concrete {
                id: id.clone(),
                args: type_args
                    .iter()
                    .map(|arg| substitute_in_type(arg, params, args))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        ),
        other => Ok(other.clone()),
    }
}

fn substitute_ports(
    ports: &Ports,
    params: &[String],
    args: &[SignalType],
) -> Result<Ports, String> {
    let mut result = Ports::new();
    for (key, port) in ports {
        result.insert(
            key.clone(),
            Port {
                id: port.id.clone(),
                signal_type: substitute_in_type(&port.signal_type, params, args)?,
                default_value: port.default_value.clone(),
            },
        );
    }
    Ok(result)
}

fn substitute_in_graph(graph: &Graph, params: &[String], args: &[SignalType]) -> Result<Graph, String> {
    let mut nodes = graph.nodes.clone();
    for node in nodes.values_mut() {
        if !node.type_args.is_empty() {
            node.type_args = node
                .type_args
                .iter()
                .map(|type_arg| substitute_in_type(type_arg, params, args))
                .collect::<Result<Vec<_>, _>>()?;
        }
    }
    Ok(Graph {
        nodes,
        edges: graph.edges.clone(),
    })
}

pub fn instantiate_node_type(
    definition: &NodeDefinition,
    type_args: &[SignalType],
) -> Result<NodeDefinition, String> {
    instantiate_node_definition(definition, type_args)
}

pub fn instantiate_node_definition(
    definition: &NodeDefinition,
    type_args: &[SignalType],
) -> Result<NodeDefinition, String> {
    let params: Vec<String> = definition
        .type_params
        .iter()
        .map(|param| param.id.clone())
        .collect();
    if params.len() != type_args.len() {
        return Err(format!(
            "expected {} typeArgs for NodeDefinition \"{}\", got {}",
            params.len(),
            definition.id,
            type_args.len()
        ));
    }
    assert_concrete_args(type_args)?;

    Ok(NodeDefinition {
        id: definition.id.clone(),
        name: definition.name.clone(),
        type_params: vec![],
        implementation: definition.implementation.clone(),
        inputs: substitute_ports(&definition.inputs, &params, type_args)?,
        outputs: substitute_ports(&definition.outputs, &params, type_args)?,
        body: match &definition.body {
            Some(body) => Some(substitute_in_graph(body, &params, type_args)?),
            None => None,
        },
        bindings: definition.bindings.clone(),
    })
}
