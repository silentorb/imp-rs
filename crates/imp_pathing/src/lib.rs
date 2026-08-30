//! Pathing NodeLibrary. Spec: imp-spec/docs/packages/imp-pathing/pathing.md

use imp_core_types::{NodeLibrary, NodeType, Port, PrimitiveValue, SignalType};
use std::collections::BTreeMap;

fn signal(id: &str) -> SignalType {
    SignalType { id: id.into() }
}

fn port(id: &str, signal_type: SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// GQL-like path operator catalog.
pub fn pathing_library() -> NodeLibrary {
    let collection = signal("collection");
    let string = signal("string");
    let number = signal("number");
    let any = signal("any");

    let traverse = NodeType {
        id: "traverse".into(),
        inputs: BTreeMap::from([
            ("collection".into(), port("collection", collection.clone(), None)),
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
            port("collection", collection, None),
        )]),
    };

    NodeLibrary {
        id: "imp.pathing".into(),
        types: BTreeMap::from([("traverse".into(), traverse)]),
    }
}
