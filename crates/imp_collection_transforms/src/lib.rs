//! Collection transform NodeLibrary.
//! Spec: imp-spec/docs/packages/imp-collection-transforms/collection-transforms.md

use imp_core_types::{
    concrete_type, list_of, set_of, type_param, type_var, NodeDefinition, NodeImplementation,
    NodeLibrary, Port, Ports, PrimitiveValue, SignalType, TypeParam,
};
use std::collections::BTreeMap;

fn port(id: &str, signal_type: SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
        variadic: false,
    }
}

fn variadic_port(id: &str, signal_type: SignalType) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value: None,
        variadic: true,
    }
}

fn type_param_t() -> Vec<TypeParam> {
    vec![type_param("T")]
}

fn list_t() -> SignalType {
    list_of(type_var("T"))
}

fn set_t() -> SignalType {
    set_of(type_var("T"))
}

fn kind_preserving(
    id: &str,
    implementation_id: &str,
    container_type: SignalType,
    extra_inputs: Ports,
) -> NodeDefinition {
    let collection_port = port("collection", container_type.clone(), None);
    let mut inputs = BTreeMap::from([("collection".into(), collection_port)]);
    inputs.extend(extra_inputs);
    NodeDefinition {
        id: id.into(),
        name: None,
        type_params: type_param_t(),
        implementation: Some(NodeImplementation::universal(implementation_id)),
        inputs,
        outputs: BTreeMap::from([(
            "collection".into(),
            port("collection", container_type, None),
        )]),
        body: None,
        bindings: None,
    }
}

fn list_and_set_overloads(
    id: &str,
    extra_inputs: Ports,
    extra_inputs_for_set: Option<Ports>,
) -> Vec<NodeDefinition> {
    let set_inputs = extra_inputs_for_set.unwrap_or_else(|| extra_inputs.clone());
    vec![
        kind_preserving(id, &format!("{id}_list"), list_t(), extra_inputs),
        kind_preserving(id, &format!("{id}_set"), set_t(), set_inputs),
    ]
}

/// Collection combinator and predicate catalog.
pub fn collection_transforms_library() -> NodeLibrary {
    let boolean = concrete_type("boolean", vec![]);
    let string = concrete_type("string", vec![]);
    let number = concrete_type("number", vec![]);
    let any = concrete_type("any", vec![]);

    let mut definitions = Vec::new();

    definitions.extend(list_and_set_overloads(
        "filter",
        BTreeMap::from([("predicate".into(), port("predicate", boolean.clone(), None))]),
        None,
    ));

    definitions.extend(list_and_set_overloads(
        "except",
        BTreeMap::from([(
            "exclude".into(),
            port("exclude", list_t(), None),
        )]),
        Some(BTreeMap::from([(
            "exclude".into(),
            port("exclude", set_t(), None),
        )])),
    ));

    definitions.push(kind_preserving(
        "sort",
        "sort",
        list_t(),
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

    definitions.push(kind_preserving(
        "limit",
        "limit",
        list_t(),
        BTreeMap::from([("count".into(), port("count", number.clone(), None))]),
    ));

    definitions.push(kind_preserving(
        "offset",
        "offset",
        list_t(),
        BTreeMap::from([("count".into(), port("count", number.clone(), None))]),
    ));

    definitions.extend(list_and_set_overloads(
        "project",
        BTreeMap::from([("columns".into(), port("columns", string.clone(), None))]),
        None,
    ));

    definitions.push(kind_preserving(
        "group",
        "group",
        list_t(),
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

    definitions.extend(list_and_set_overloads(
        "search",
        BTreeMap::from([("query".into(), port("query", string.clone(), None))]),
        None,
    ));

    definitions.push(NodeDefinition {
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
        definitions.push(NodeDefinition {
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

    definitions.push(NodeDefinition {
        name: None,
        implementation: None,
        id: "and".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("operands".into(), variadic_port("operands", boolean.clone()))]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        body: None,
        bindings: None,
    });

    definitions.push(NodeDefinition {
        name: None,
        implementation: None,
        id: "or".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("operands".into(), variadic_port("operands", boolean.clone()))]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        body: None,
        bindings: None,
    });

    definitions.push(NodeDefinition {
        name: None,
        implementation: None,
        id: "not".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("value".into(), port("value", boolean.clone(), None))]),
        outputs: BTreeMap::from([("value".into(), port("value", boolean, None))]),
        body: None,
        bindings: None,
    });

    definitions.push(NodeDefinition {
        name: None,
        implementation: None,
        id: "column".into(),
        type_params: vec![],
        inputs: BTreeMap::from([("name".into(), port("name", string.clone(), None))]),
        outputs: BTreeMap::from([("value".into(), port("value", any.clone(), None))]),
        body: None,
        bindings: None,
    });

    definitions.push(NodeDefinition {
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
