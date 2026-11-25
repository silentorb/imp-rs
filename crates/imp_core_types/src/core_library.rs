//! Core boundary node types. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use crate::{
    concrete_type, list_of, type_param, type_var, NodeDefinition, NodeLibrary, Port, Ports,
    PrimitiveValue, Type, TypeParam,
};
use std::collections::BTreeMap;

fn port(id: &str, ty: Type, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        ty,
        default_value,
        variadic: false,
    }
}

fn variadic_port(id: &str, ty: Type) -> Port {
    Port {
        id: id.into(),
        ty,
        default_value: None,
        variadic: true,
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
                outputs: BTreeMap::from([("value".into(), port("value", t.clone(), None))]),
                body: None,
                bindings: None,
            },
            NodeDefinition {
                id: "pack".into(),
                name: None,
                type_params: type_param_t(),
                implementation: None,
                inputs: BTreeMap::from([("items".into(), variadic_port("items", t.clone()))]),
                outputs: BTreeMap::from([("list".into(), port("list", list_of(t), None))]),
                body: None,
                bindings: None,
            },
        ],
    }
}
