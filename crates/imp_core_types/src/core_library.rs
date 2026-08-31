//! Core boundary node types. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use crate::{
    concrete_type, type_var, NodeDefinition, NodeLibrary, Port, Ports, PrimitiveValue, SignalType,
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

    NodeLibrary {
        id: "imp.core".into(),
        definitions: vec![
            NodeDefinition {
                id: "input".into(),
                name: None,
                type_params: type_param_t(),
                implementation: None,
                inputs: Ports::new(),
                outputs: BTreeMap::from([("value".into(), port("value", t.clone(), None))]),
                body: None,
                bindings: None,
            },
            NodeDefinition {
                id: "output".into(),
                name: None,
                type_params: type_param_t(),
                implementation: None,
                inputs: BTreeMap::from([("value".into(), port("value", t.clone(), None))]),
                outputs: Ports::new(),
                body: None,
                bindings: None,
            },
            NodeDefinition {
                id: "parameter".into(),
                name: None,
                type_params: type_param_t(),
                implementation: None,
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
                body: None,
                bindings: None,
            },
        ],
    }
}
