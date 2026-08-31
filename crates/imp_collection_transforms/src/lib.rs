//! Collection transform NodeLibrary.
//! Spec: imp-spec/docs/packages/imp-collection-transforms/collection-transforms.md

use imp_core_types::{
    collection_of, concrete_type, type_var, NodeImplementation, NodeLibrary, NodeType, Port, Ports,
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

fn type_param_t() -> Vec<TypeParam> {
    vec![TypeParam { id: "T".into() }]
}

fn collection_t() -> SignalType {
    collection_of(type_var("T"))
}

fn collection_in() -> Port {
    port("collection", collection_t(), None)
}

fn collection_out() -> Port {
    port("collection", collection_t(), None)
}

fn collection_preserving(id: &str, extra_inputs: Ports) -> NodeType {
    let mut inputs = BTreeMap::from([("collection".into(), collection_in())]);
    inputs.extend(extra_inputs);
    NodeType {
        id: id.into(),
        type_params: type_param_t(),
        implementation: Some(NodeImplementation::universal(id)),
        inputs,
        outputs: BTreeMap::from([("collection".into(), collection_out())]),
    }
}

/// Collection combinator and predicate catalog.
pub fn collection_transforms_library() -> NodeLibrary {
    let collection_t_sig = collection_t();
    let boolean = concrete_type("boolean", vec![]);
    let string = concrete_type("string", vec![]);
    let number = concrete_type("number", vec![]);
    let any = concrete_type("any", vec![]);

    let mut types = BTreeMap::new();

    let mut add = |id: &str, node_type: NodeType| {
        types.insert(id.into(), node_type);
    };

    add(
        "filter",
        collection_preserving(
            "filter",
            BTreeMap::from([("predicate".into(), port("predicate", boolean.clone(), None))]),
        ),
    );

    add(
        "except",
        collection_preserving(
            "except",
            BTreeMap::from([(
                "exclude".into(),
                port("exclude", collection_t_sig.clone(), None),
            )]),
        ),
    );

    add(
        "sort",
        collection_preserving(
            "sort",
            BTreeMap::from([
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
        ),
    );

    add(
        "limit",
        collection_preserving(
            "limit",
            BTreeMap::from([("count".into(), port("count", number.clone(), None))]),
        ),
    );

    add(
        "offset",
        collection_preserving(
            "offset",
            BTreeMap::from([("count".into(), port("count", number.clone(), None))]),
        ),
    );

    add(
        "project",
        collection_preserving(
            "project",
            BTreeMap::from([("columns".into(), port("columns", string.clone(), None))]),
        ),
    );

    add(
        "group",
        collection_preserving(
            "group",
            BTreeMap::from([
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
        ),
    );

    add(
        "search",
        collection_preserving(
            "search",
            BTreeMap::from([("query".into(), port("query", string.clone(), None))]),
        ),
    );

    add(
        "contains",
        NodeType {
            implementation: None,
            id: "contains".into(),
            type_params: vec![],
            inputs: BTreeMap::from([
                ("haystack".into(), port("haystack", any.clone(), None)),
                ("needle".into(), port("needle", string.clone(), None)),
            ]),
            outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        },
    );

    for name in ["equals", "not_equals", "less_than", "greater_than"] {
        add(
            name,
            NodeType {
                implementation: None,
                id: name.into(),
                type_params: vec![],
                inputs: BTreeMap::from([
                    ("left".into(), port("left", any.clone(), None)),
                    ("right".into(), port("right", any.clone(), None)),
                ]),
                outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
            },
        );
    }

    add(
        "and",
        NodeType {
            implementation: None,
            id: "and".into(),
            type_params: vec![],
            inputs: BTreeMap::from([
                ("left".into(), port("left", boolean.clone(), None)),
                ("right".into(), port("right", boolean.clone(), None)),
            ]),
            outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        },
    );

    add(
        "or",
        NodeType {
            implementation: None,
            id: "or".into(),
            type_params: vec![],
            inputs: BTreeMap::from([
                ("left".into(), port("left", boolean.clone(), None)),
                ("right".into(), port("right", boolean.clone(), None)),
            ]),
            outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        },
    );

    add(
        "not",
        NodeType {
            implementation: None,
            id: "not".into(),
            type_params: vec![],
            inputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
            outputs: BTreeMap::from([("value".into(), port("value", boolean, None))]),
        },
    );

    add(
        "column",
        NodeType {
            implementation: None,
            id: "column".into(),
            type_params: vec![],
            inputs: BTreeMap::from([("name".into(), port("name", string.clone(), None))]),
            outputs: BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
        },
    );

    add(
        "literal",
        NodeType {
            implementation: None,
            id: "literal".into(),
            type_params: vec![],
            inputs: BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
            outputs: BTreeMap::from([("value".into(), port("value", any, None))]),
        },
    );

    NodeLibrary {
        id: "imp.collection.transforms".into(),
        types,
    }
}
