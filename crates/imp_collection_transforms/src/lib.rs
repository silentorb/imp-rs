//! Collection transform NodeLibrary.
//! Spec: imp-spec/docs/packages/imp-collection-transforms/collection-transforms.md

use imp_core_types::{
    collection_of, concrete_type, type_param, type_var, NodeDefinition, NodeImplementation,
    NodeLibrary, Port, Ports, PrimitiveValue, SignalType, TypeParam,
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

fn collection_t() -> SignalType {
    collection_of(type_var("T"))
}

fn collection_in() -> Port {
    port("collection", collection_t(), None)
}

fn collection_out() -> Port {
    port("collection", collection_t(), None)
}

fn collection_preserving(id: &str, extra_inputs: Ports) -> NodeDefinition {
    let mut inputs = BTreeMap::from([("collection".into(), collection_in())]);
    inputs.extend(extra_inputs);
    NodeDefinition {
        id: id.into(),
        name: None,
        type_params: type_param_t(),
        implementation: Some(NodeImplementation::universal(id)),
        inputs,
        outputs: BTreeMap::from([("collection".into(), collection_out())]),
        body: None,
        bindings: None,
    }
}

/// Collection combinator and predicate catalog.
pub fn collection_transforms_library() -> NodeLibrary {
    let collection_t_sig = collection_t();
    let boolean = concrete_type("boolean", vec![]);
    let string = concrete_type("string", vec![]);
    let number = concrete_type("number", vec![]);
    let any = concrete_type("any", vec![]);

    let mut definitions = Vec::new();

    let mut add = |node_type: NodeDefinition| {
        definitions.push(node_type);
    };

    add(collection_preserving(
        "filter",
        BTreeMap::from([("predicate".into(), port("predicate", boolean.clone(), None))]),
    ));

    add(collection_preserving(
        "except",
        BTreeMap::from([(
            "exclude".into(),
            port("exclude", collection_t_sig.clone(), None),
        )]),
    ));

    add(collection_preserving(
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
    ));

    add(collection_preserving(
        "limit",
        BTreeMap::from([("count".into(), port("count", number.clone(), None))]),
    ));

    add(collection_preserving(
        "offset",
        BTreeMap::from([("count".into(), port("count", number.clone(), None))]),
    ));

    add(collection_preserving(
        "project",
        BTreeMap::from([("columns".into(), port("columns", string.clone(), None))]),
    ));

    add(collection_preserving(
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
    ));

    add(collection_preserving(
        "search",
        BTreeMap::from([("query".into(), port("query", string.clone(), None))]),
    ));

    add(NodeDefinition {
        name: None,
        implementation: None,
        id: "contains".into(),
        type_params: vec![],
        inputs: BTreeMap::from([
            ("haystack".into(), port("haystack", any.clone(), None)),
            ("needle".into(), port("needle", string.clone(), None)),
        ]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        body: None,
        bindings: None,
    });

    for name in ["equals", "not_equals", "less_than", "greater_than"] {
        add(NodeDefinition {
            name: None,
            implementation: None,
            id: name.into(),
            type_params: vec![],
            inputs: BTreeMap::from([
                ("left".into(), port("left", any.clone(), None)),
                ("right".into(), port("right", any.clone(), None)),
            ]),
            outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
            body: None,
            bindings: None,
        });
    }

    add(NodeDefinition {
        name: None,
        implementation: None,
        id: "and".into(),
        type_params: vec![],
        inputs: BTreeMap::from([
            ("left".into(), port("left", boolean.clone(), None)),
            ("right".into(), port("right", boolean.clone(), None)),
        ]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        body: None,
        bindings: None,
    });

    add(NodeDefinition {
        name: None,
        implementation: None,
        id: "or".into(),
        type_params: vec![],
        inputs: BTreeMap::from([
            ("left".into(), port("left", boolean.clone(), None)),
            ("right".into(), port("right", boolean.clone(), None)),
        ]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        body: None,
        bindings: None,
    });

    add(NodeDefinition {
        name: None,
        implementation: None,
        id: "not".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean, None))]),
        body: None,
        bindings: None,
    });

    add(NodeDefinition {
        name: None,
        implementation: None,
        id: "column".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("name".into(), port("name", string.clone(), None))]),
        outputs: BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
        body: None,
        bindings: None,
    });

    add(NodeDefinition {
        name: None,
        implementation: None,
        id: "literal".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
        outputs: BTreeMap::from([("value".into(), port("value", any, None))]),
        body: None,
        bindings: None,
    });

    NodeLibrary {
        id: "imp.collection.transforms".into(),
        definitions,
    }
}
