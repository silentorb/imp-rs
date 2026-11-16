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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![list_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
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
                order: None,
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
                order: None,
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![list_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "lim".into(),
                Node {
                    id: "lim".into(),
                    title: None,
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
                order: None,
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
                order: None,
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
                order: None,
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![set_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "lim".into(),
                Node {
                    id: "lim".into(),
                    title: None,
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
                order: None,
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
                order: None,
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
                order: None,
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![set_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "p".into(),
                Node {
                    id: "p".into(),
                    title: None,
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
                order: None,
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
                order: None,
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
                order: None,
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![collection_of(concrete_type("any", vec![]))],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
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
                order: None,
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
                order: None,
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![concrete_type("any", vec![])],
                    inputs: Default::default(),
                },
            ),
            (
                "pred".into(),
                Node {
                    id: "pred".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    title: None,
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
                order: None,
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
                order: None,
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

#[test]
fn accepts_nary_and_with_unique_edge_order() {
    let registry = transforms_registry();
    let graph = Graph {
        nodes: [
            (
                "a".into(),
                Node {
                    id: "a".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "b".into(),
                Node {
                    id: "b".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(false))].into(),
                },
            ),
            (
                "c".into(),
                Node {
                    id: "c".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "and1".into(),
                Node {
                    id: "and1".into(),
                    title: None,
                    node_type: "and".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [
            (
                "e0".into(),
                Edge {
                    from: PortReference {
                        node: "a".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "and1".into(),
                        port: "operands".into(),
                    },
                    order: Some(0),
                },
            ),
            (
                "e1".into(),
                Edge {
                    from: PortReference {
                        node: "b".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "and1".into(),
                        port: "operands".into(),
                    },
                    order: Some(1),
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "c".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "and1".into(),
                        port: "operands".into(),
                    },
                    order: Some(2),
                },
            ),
        ]
        .into(),
    };
    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn rejects_missing_edge_order_on_variadic_target() {
    let registry = transforms_registry();
    let graph = Graph {
        nodes: [
            (
                "a".into(),
                Node {
                    id: "a".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "and1".into(),
                Node {
                    id: "and1".into(),
                    title: None,
                    node_type: "and".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [(
            "e0".into(),
            Edge {
                from: PortReference {
                    node: "a".into(),
                    port: "value".into(),
                },
                to: PortReference {
                    node: "and1".into(),
                    port: "operands".into(),
                },
                order: None,
            },
        )]
        .into(),
    };
    let errors = check_graph(&graph, &registry);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("requires Edge.order")));
}

#[test]
fn rejects_duplicate_edge_order_on_variadic_target() {
    let registry = transforms_registry();
    let graph = Graph {
        nodes: [
            (
                "a".into(),
                Node {
                    id: "a".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "b".into(),
                Node {
                    id: "b".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(false))].into(),
                },
            ),
            (
                "and1".into(),
                Node {
                    id: "and1".into(),
                    title: None,
                    node_type: "and".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [
            (
                "e0".into(),
                Edge {
                    from: PortReference {
                        node: "a".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "and1".into(),
                        port: "operands".into(),
                    },
                    order: Some(0),
                },
            ),
            (
                "e1".into(),
                Edge {
                    from: PortReference {
                        node: "b".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "and1".into(),
                        port: "operands".into(),
                    },
                    order: Some(0),
                },
            ),
        ]
        .into(),
    };
    let errors = check_graph(&graph, &registry);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("duplicate Edge.order")));
}

#[test]
fn rejects_multiple_edges_on_non_variadic_input() {
    let registry = transforms_registry();
    let graph = Graph {
        nodes: [
            (
                "a".into(),
                Node {
                    id: "a".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(true))].into(),
                },
            ),
            (
                "b".into(),
                Node {
                    id: "b".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: [("value".into(), PrimitiveValue::Bool(false))].into(),
                },
            ),
            (
                "not1".into(),
                Node {
                    id: "not1".into(),
                    title: None,
                    node_type: "not".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into(),
        edges: [
            (
                "e0".into(),
                Edge {
                    from: PortReference {
                        node: "a".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "not1".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
            (
                "e1".into(),
                Edge {
                    from: PortReference {
                        node: "b".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "not1".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]
        .into(),
    };
    let errors = check_graph(&graph, &registry);
    assert!(errors.iter().any(|e| e.message.contains("non-variadic")));
}
