//! Pathing NodeLibrary. Spec: imp-spec/docs/packages/imp-pathing/pathing.md

use imp_core_types::{
    collection_of, concrete_type, type_param, type_var, NodeDefinition, NodeImplementation,
    NodeLibrary, Port, PrimitiveValue, Type,
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

/// GQL-like path operator catalog.
pub fn pathing_library() -> NodeLibrary {
    let collection_t = collection_of(type_var("T"));
    let string = concrete_type("string", vec![]);
    let number = concrete_type("number", vec![]);
    let any = concrete_type("any", vec![]);

    NodeLibrary {
        types: Vec::new(),
        id: "imp.pathing".into(),
        definitions: vec![NodeDefinition {
            id: "traverse".into(),
            name: None,
            type_params: vec![type_param("T")],
            implementation: Some(NodeImplementation::universal("traverse")),
            inputs: BTreeMap::from([
                ("collection".into(), port("collection", collection_t.clone(), None)),
                ("association".into(), port("association", string.clone(), None)),
                (
                    "direction".into(),
                    port("direction", number.clone(), Some(PrimitiveValue::Number(0.0))),
                ),
                (
                    "edge_property".into(),
                    port("edge_property", string.clone(), Some(PrimitiveValue::Null)),
                ),
                (
                    "edge_equals".into(),
                    port("edge_equals", any.clone(), Some(PrimitiveValue::Null)),
                ),
            ]),
            outputs: BTreeMap::from([(
                "collection".into(),
                port("collection", collection_t, None),
            )]),
            body: None,
            bindings: None,
        }],
    }
}
