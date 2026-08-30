//! Collection transform NodeLibrary.
//! Spec: imp-spec/docs/packages/imp-collection-transforms/collection-transforms.md

use imp_core_types::{NodeLibrary, NodeType, Port, Ports, PrimitiveValue, SignalType};
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

fn collection_in() -> Port {
    port("collection", signal("collection"), None)
}

fn collection_out() -> Port {
    port("collection", signal("collection"), None)
}

/// Collection combinator and predicate catalog.
pub fn collection_transforms_library() -> NodeLibrary {
    let collection = signal("collection");
    let boolean = signal("boolean");
    let string = signal("string");
    let number = signal("number");
    let any = signal("any");

    let mut types = BTreeMap::new();

    let mut add = |id: &str, inputs: Ports, outputs: Ports| {
        types.insert(
            id.into(),
            NodeType {
                id: id.into(),
                inputs,
                outputs,
            },
        );
    };

    add(
        "filter",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("predicate".into(), port("predicate", boolean.clone(), None)),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "except",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("exclude".into(), port("exclude", collection.clone(), None)),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "sort",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("column".into(), port("column", string.clone(), None)),
            (
                "direction".into(),
                port(
                    "direction",
                    string.clone(),
                    Some(PrimitiveValue::String("asc".into())),
                ),
            ),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "limit",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("count".into(), port("count", number.clone(), None)),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "offset",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("count".into(), port("count", number.clone(), None)),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "project",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("columns".into(), port("columns", string.clone(), None)),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "group",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("column".into(), port("column", string.clone(), None)),
            (
                "direction".into(),
                port(
                    "direction",
                    string.clone(),
                    Some(PrimitiveValue::String("asc".into())),
                ),
            ),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "search",
        BTreeMap::from([
            ("collection".into(), collection_in()),
            ("query".into(), port("query", string.clone(), None)),
        ]),
        BTreeMap::from([("collection".into(), collection_out())]),
    );

    add(
        "contains",
        BTreeMap::from([
            ("haystack".into(), port("haystack", any.clone(), None)),
            ("needle".into(), port("needle", string.clone(), None)),
        ]),
        BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
    );

    for name in ["equals", "not_equals", "less_than", "greater_than"] {
        add(
            name,
            BTreeMap::from([
                ("left".into(), port("left", any.clone(), None)),
                ("right".into(), port("right", any.clone(), None)),
            ]),
            BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        );
    }

    add(
        "and",
        BTreeMap::from([
            ("left".into(), port("left", boolean.clone(), None)),
            ("right".into(), port("right", boolean.clone(), None)),
        ]),
        BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
    );

    add(
        "or",
        BTreeMap::from([
            ("left".into(), port("left", boolean.clone(), None)),
            ("right".into(), port("right", boolean.clone(), None)),
        ]),
        BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
    );

    add(
        "not",
        BTreeMap::from([(
            "value".into(),
            port("value", boolean.clone(), None),
        )]),
        BTreeMap::from([("value".into(), port("value", boolean, None))]),
    );

    add(
        "column",
        BTreeMap::from([("name".into(), port("name", string.clone(), None))]),
        BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
    );

    add(
        "literal",
        BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
        BTreeMap::from([("value".into(), port("value", any, None))]),
    );

    NodeLibrary {
        id: "imp.collection.transforms".into(),
        types,
    }
}
