//! NodeType instantiation.

use imp_core_types::{NodeType, Port, Ports, SignalType};

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

pub fn instantiate_node_type(
    node_type: &NodeType,
    type_args: &[SignalType],
) -> Result<NodeType, String> {
    let params: Vec<String> = node_type
        .type_params
        .iter()
        .map(|param| param.id.clone())
        .collect();
    if params.len() != type_args.len() {
        return Err(format!(
            "expected {} typeArgs for NodeType \"{}\", got {}",
            params.len(),
            node_type.id,
            type_args.len()
        ));
    }
    assert_concrete_args(type_args)?;

    Ok(NodeType {
        implementation: None,
        id: node_type.id.clone(),
        type_params: vec![],
        inputs: substitute_ports(&node_type.inputs, &params, type_args)?,
        outputs: substitute_ports(&node_type.outputs, &params, type_args)?,
    })
}
