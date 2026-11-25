//! Assignability and subtype matching tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::{
    collection_of, concrete_type, core_node_library, list_of, set_of, Edge, Graph, Node,
    PortReference, PrimitiveValue,
};
use imp_registry::{create_registry, load_library};
use imp_typecheck::{assignable, check_graph, type_matches, unify, Substitution};

#[test]
fn list_is_assignable_to_collection_with_matching_args() {
    let result = assignable(
        &list_of(concrete_type("string", vec![])),
        &collection_of(concrete_type("string", vec![])),
        &Substitution::new(),
    );
    assert!(result.is_ok());
}

#[test]
fn collection_is_not_assignable_to_list() {
    let result = assignable(
        &collection_of(concrete_type("string", vec![])),
        &list_of(concrete_type("string", vec![])),
        &Substitution::new(),
    );
    assert!(result.is_err());
}

#[test]
fn unify_does_not_equate_list_and_collection() {
    let result = unify(
        &list_of(concrete_type("string", vec![])),
        &collection_of(concrete_type("string", vec![])),
        &Substitution::new(),
    );
    assert!(result.is_err());
}

#[test]
fn type_matches_allows_subtype_against_supertype_pattern() {
    assert!(type_matches(
        &collection_of(concrete_type("string", vec![])),
        &list_of(concrete_type("string", vec![])),
    ));
    assert!(!type_matches(
        &list_of(concrete_type("string", vec![])),
        &collection_of(concrete_type("string", vec![])),
    ));
}

#[test]
fn set_is_assignable_to_collection_with_matching_args() {
    let result = assignable(
        &set_of(concrete_type("string", vec![])),
        &collection_of(concrete_type("string", vec![])),
        &Substitution::new(),
    );
    assert!(result.is_ok());
}

#[test]
fn set_is_not_assignable_to_list() {
    let result = assignable(
        &set_of(concrete_type("string", vec![])),
        &list_of(concrete_type("string", vec![])),
        &Substitution::new(),
    );
    assert!(result.is_err());
}

#[test]
fn accepts_list_input_wired_to_filter_list_overload() {
    let registry = load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        collection_transforms_library(),
    )
    .unwrap();

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
fn rejects_collection_input_wired_to_list_sort_port() {
    let registry = load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        collection_transforms_library(),
    )
    .unwrap();

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
                "sort".into(),
                Node {
                    id: "sort".into(),
                    title: None,
                    node_type: "sort".into(),
                    type_args: vec![],
                    inputs: [
                        ("column".into(), PrimitiveValue::String("name".into())),
                        ("direction".into(), PrimitiveValue::String("asc".into())),
                    ]
                    .into(),
                },
            ),
        ]
        .into(),
        edges: [(
            "e1".into(),
            Edge {
                from: PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                to: PortReference {
                    node: "sort".into(),
                    port: "collection".into(),
                },
            order: None,
            },
        )]
        .into(),
    };

    let errors = check_graph(&graph, &registry);
    assert!(errors.iter().any(|e| e.port_id.as_deref() == Some("collection")));
}
