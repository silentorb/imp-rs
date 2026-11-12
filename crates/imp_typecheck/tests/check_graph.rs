//! imp-typecheck integration tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::{
    collection_of, concrete_type, core_node_library, list_of, set_of, Edge, Graph, Node,
    PortReference, PrimitiveValue,
};
use imp_registry::{create_registry, load_library};
use imp_typecheck::check_graph;

fn transforms_registry() -> imp_registry::Registry {
    load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        collection_transforms_library(),
    )
    .unwrap()
}

#[test]
fn accepts_list_input_wired_to_filter_list_overload() {
    let registry = transforms_registry();

    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![list_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [
            (
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
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "pred".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "f".into(),
                        port: "predicate".into(),
                    },
                },
            ),
        ]
        .into(),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn accepts_list_filter_limit_via_list_overload() {
    let registry = transforms_registry();

    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![list_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "lim".into(),
                Node {
                    id: "lim".into(),
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: [("count".into(), PrimitiveValue::Number(10.0))].into(),
                },
            ),
        ]
        .into(),
        edges: [
            (
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
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "pred".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "f".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e3".into(),
                Edge {
                    from: PortReference {
                        node: "f".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "lim".into(),
                        port: "collection".into(),
                    },
                },
            ),
        ]
        .into(),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn rejects_set_filter_limit() {
    let registry = transforms_registry();

    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![set_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "lim".into(),
                Node {
                    id: "lim".into(),
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: [("count".into(), PrimitiveValue::Number(10.0))].into(),
                },
            ),
        ]
        .into(),
        edges: [
            (
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
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "pred".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "f".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e3".into(),
                Edge {
                    from: PortReference {
                        node: "f".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "lim".into(),
                        port: "collection".into(),
                    },
                },
            ),
        ]
        .into(),
    };

    assert!(!check_graph(&graph, &registry).is_empty());
}

#[test]
fn accepts_set_filter_project_via_set_overload() {
    let registry = transforms_registry();

    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![set_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "p".into(),
                Node {
                    id: "p".into(),
                    node_type: "project".into(),
                    type_args: vec![],
                    inputs: [("columns".into(), PrimitiveValue::String("id".into()))].into(),
                },
            ),
        ]
        .into(),
        edges: [
            (
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
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "pred".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "f".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e3".into(),
                Edge {
                    from: PortReference {
                        node: "f".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "p".into(),
                        port: "collection".into(),
                    },
                },
            ),
        ]
        .into(),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn rejects_collection_input_wired_to_filter() {
    let registry = transforms_registry();

    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![collection_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [
            (
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
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "pred".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "f".into(),
                        port: "predicate".into(),
                    },
                },
            ),
        ]
        .into(),
    };

    let errors = check_graph(&graph, &registry);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("no matching overload")));
}

#[test]
fn reports_ambiguous_filter_when_input_is_any() {
    let registry = transforms_registry();

    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![concrete_type("any", vec![])],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [
            (
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
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "pred".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "f".into(),
                        port: "predicate".into(),
                    },
                },
            ),
        ]
        .into(),
    };

    let errors = check_graph(&graph, &registry);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("ambiguous overload")));
}
