//! Pathing NodeLibrary. Spec: imp-spec/docs/packages/imp-pathing/pathing.md

use imp_core_types::{
    collection_of, concrete_type, type_var, NodeImplementation, NodeLibrary, NodeType, Port,
    PrimitiveValue, SignalType, TypeParam,
};
use std::collections::BTreeMap;

fn port(id: &str, signal_type: SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// GQL-like path operator catalog.
pub fn pathing_library() -> NodeLibrary {
    let collection_t = collection_of(type_var("T"));
    let string = concrete_type("string", vec![]);
    let number = concrete_type("number", vec![]);
    let any = concrete_type("any", vec![]);

    let traverse = NodeType {
        id: "traverse".into(),
        type_params: vec![TypeParam { id: "T".into() }],
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
                port(
                    "edge_property",
                    string.clone(),
                    Some(PrimitiveValue::Null),
                ),
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
    };

    NodeLibrary {
        id: "imp.pathing".into(),
        types: BTreeMap::from([("traverse".into(), traverse)]),
    }
}
