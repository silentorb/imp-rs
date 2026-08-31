//! Core boundary node types. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use crate::{
    concrete_type, type_var, NodeLibrary, NodeType, Port, Ports, PrimitiveValue, SignalType,
    TypeParam,
    type_param,
};
use std::collections::BTreeMap;

fn port(id: &str, signal_type: SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

fn type_param_t() -> Vec<TypeParam> {
    vec![type_param("T")]
}

/// Core Imp boundary nodes — one instance per external port.
pub fn core_node_library() -> NodeLibrary {
    let t = type_var("T");
    let string = concrete_type("string", vec![]);
    let any = concrete_type("any", vec![]);

    let mut types = BTreeMap::new();

    types.insert(
        "input".into(),
        NodeType {
            implementation: None,
            id: "input".into(),
            type_params: type_param_t(),
            inputs: Ports::new(),
            outputs: BTreeMap::from([("value".into(), port("value", t.clone(), None))]),
        },
    );

    types.insert(
        "output".into(),
        NodeType {
            implementation: None,
            id: "output".into(),
            type_params: type_param_t(),
            inputs: BTreeMap::from([("value".into(), port("value", t.clone(), None))]),
            outputs: Ports::new(),
        },
    );

    types.insert(
        "parameter".into(),
        NodeType {
            implementation: None,
            id: "parameter".into(),
            type_params: type_param_t(),
            inputs: BTreeMap::from([
                (
                    "label".into(),
                    port(
                        "label",
                        string.clone(),
                        Some(PrimitiveValue::String(String::new())),
                    ),
                ),
                (
                    "value".into(),
                    port("value", any.clone(), Some(PrimitiveValue::Null)),
                ),
            ]),
            outputs: BTreeMap::from([("value".into(), port("value", t, None))]),
        },
    );

    NodeLibrary {
        id: "imp.core".into(),
        types,
    }
}
