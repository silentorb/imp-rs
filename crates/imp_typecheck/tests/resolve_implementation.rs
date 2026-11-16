//! Implementation dispatch tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::{
    concrete_type, core_node_library, list_of, set_of, Edge, Graph, ImplementationCase, Node,
    NodeImplementation, NodeLibrary, NodeType, Port, PortReference, SignalType, TypeParam,
};
use imp_registry::{create_registry, load_library};
use imp_typecheck::{resolve_implementation, resolve_type_args};
use std::collections::BTreeMap;

fn cast_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.cast".into(),
        definitions: vec![NodeType {
            id: "cast".into(),
            name: None,
            type_params: vec![TypeParam {
                id: "T".into(),
                bounds: vec![],
            }],
            body: None,
            bindings: None,
            implementation: Some(NodeImplementation::Dispatch {
                param_index: None,
                cases: vec![
                    ImplementationCase {
                        match_type: concrete_type("string", vec![]),
                        implementation: "cast_string".into(),
                    },
                    ImplementationCase {
                        match_type: concrete_type("number", vec![]),
                        implementation: "cast_number".into(),
                    },
                ],
                default: Some("cast_any".into()),
            }),
            inputs: BTreeMap::from([(
                "value".into(),
                Port {
                    id: "value".into(),
                    signal_type: SignalType::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
            outputs: BTreeMap::from([(
                "value".into(),
                Port {
                    id: "value".into(),
                    signal_type: concrete_type("string", vec![]),
                    default_value: None,
                variadic: false,
                },
            )]),
        }],
    }
}

#[test]
fn resolve_implementation_uses_selected_list_overload_id() {
    let registry = load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        collection_transforms_library(),
    )
    .unwrap();

    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![list_of(concrete_type("any", vec![]))],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([(
            "e1".into(),
            Edge {
                from: PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                to: PortReference {
                    node: "f".into(),
                    port: "collection".into(),
                },
            order: None,
            },
        )]),
    };

    assert_eq!(
        resolve_implementation(&graph, "f", &registry).unwrap(),
        "filter_list"
    );
}

#[test]
fn resolve_implementation_uses_selected_set_overload_id() {
    let registry = load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        collection_transforms_library(),
    )
    .unwrap();

    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![set_of(concrete_type("any", vec![]))],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([(
            "e1".into(),
            Edge {
                from: PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                to: PortReference {
                    node: "f".into(),
                    port: "collection".into(),
                },
            order: None,
            },
        )]),
    };

    assert_eq!(
        resolve_implementation(&graph, "f", &registry).unwrap(),
        "filter_set"
    );
}

#[test]
fn resolve_implementation_dispatches_on_inferred_type_arg() {
    let registry = load_library(
        load_library(
            load_library(create_registry(), core_node_library()).unwrap(),
            cast_library(),
        )
        .unwrap(),
        collection_transforms_library(),
    )
    .unwrap();

    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![concrete_type("number", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "c".into(),
                Node {
                    id: "c".into(),
                    title: None,
                    node_type: "cast".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([(
            "e1".into(),
            Edge {
                from: PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                to: PortReference {
                    node: "c".into(),
                    port: "value".into(),
                },
            order: None,
            },
        )]),
    };

    assert_eq!(
        resolve_type_args(&graph, &"c".into(), &registry, None).unwrap(),
        vec![concrete_type("number", vec![])]
    );
    assert_eq!(
        resolve_implementation(&graph, "c", &registry).unwrap(),
        "cast_number"
    );
}
