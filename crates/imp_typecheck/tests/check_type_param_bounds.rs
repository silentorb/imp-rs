//! Type-parameter bound checking tests.

use imp_core_types::{
    concrete_type, core_node_library, Edge, Graph, Node, NodeLibrary, NodeType, Port,
    PortReference, SignalType, TypeConstraint, TypeConstraintLibrary, TypeParam,
};
use imp_registry::{create_registry, load_library, load_type_constraint_library};
use imp_typecheck::check_graph;
use std::collections::BTreeMap;

fn scalar_constraint_library() -> TypeConstraintLibrary {
    TypeConstraintLibrary {
        id: "example.constraints".into(),
        constraints: BTreeMap::from([(
            "Scalar".into(),
            TypeConstraint {
                id: "Scalar".into(),
                members: vec![
                    concrete_type("string", vec![]),
                    concrete_type("number", vec![]),
                ],
            },
        )]),
    }
}

fn identity_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.identity".into(),
        types: BTreeMap::from([(
            "identity".into(),
            NodeType {
                id: "identity".into(),
                type_params: vec![TypeParam {
                    id: "T".into(),
                    bounds: vec!["Scalar".into()],
                }],
                inputs: BTreeMap::from([(
                    "value".into(),
                    Port {
                        id: "value".into(),
                        signal_type: SignalType::Var { param: "T".into() },
                        default_value: None,
                    },
                )]),
                outputs: BTreeMap::from([(
                    "value".into(),
                    Port {
                        id: "value".into(),
                        signal_type: SignalType::Var { param: "T".into() },
                        default_value: None,
                    },
                )]),
                implementation: None,
            },
        )]),
    }
}

fn registry_with_bounds() -> imp_registry::Registry {
    load_type_constraint_library(
        load_library(
            load_library(create_registry(), core_node_library()).unwrap(),
            identity_library(),
        )
        .unwrap(),
        scalar_constraint_library(),
    )
    .unwrap()
}

#[test]
fn check_graph_accepts_type_arg_that_satisfies_bound() {
    let registry = registry_with_bounds();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![concrete_type("number", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "id".into(),
                Node {
                    id: "id".into(),
                    node_type: "identity".into(),
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
                    node: "id".into(),
                    port: "value".into(),
                },
            },
        )]),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn check_graph_rejects_type_arg_that_violates_bound() {
    let registry = registry_with_bounds();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![concrete_type("boolean", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "id".into(),
                Node {
                    id: "id".into(),
                    node_type: "identity".into(),
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
                    node: "id".into(),
                    port: "value".into(),
                },
            },
        )]),
    };

    let errors = check_graph(&graph, &registry);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("does not satisfy bound \"Scalar\""))
    );
}

#[test]
fn check_graph_any_satisfies_bound() {
    let registry = registry_with_bounds();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![concrete_type("any", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "id".into(),
                Node {
                    id: "id".into(),
                    node_type: "identity".into(),
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
                    node: "id".into(),
                    port: "value".into(),
                },
            },
        )]),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn check_graph_reports_unknown_bound_id() {
    let bad_library = NodeLibrary {
        id: "example.bad".into(),
        types: BTreeMap::from([(
            "bad".into(),
            NodeType {
                id: "bad".into(),
                type_params: vec![TypeParam {
                    id: "T".into(),
                    bounds: vec!["Missing".into()],
                }],
                inputs: BTreeMap::from([(
                    "value".into(),
                    Port {
                        id: "value".into(),
                        signal_type: SignalType::Var { param: "T".into() },
                        default_value: None,
                    },
                )]),
                outputs: BTreeMap::from([(
                    "value".into(),
                    Port {
                        id: "value".into(),
                        signal_type: SignalType::Var { param: "T".into() },
                        default_value: None,
                    },
                )]),
                implementation: None,
            },
        )]),
    };

    let registry = load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        bad_library,
    )
    .unwrap();

    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![concrete_type("number", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "bad".into(),
                Node {
                    id: "bad".into(),
                    node_type: "bad".into(),
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
                    node: "bad".into(),
                    port: "value".into(),
                },
            },
        )]),
    };

    let errors = check_graph(&graph, &registry);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("unknown type constraint \"Missing\""))
    );
}

#[test]
fn check_graph_matches_parametric_member_patterns() {
    let collection_constraint_library = TypeConstraintLibrary {
        id: "example.collection-constraints".into(),
        constraints: BTreeMap::from([(
            "StringCollection".into(),
            TypeConstraint {
                id: "StringCollection".into(),
                members: vec![concrete_type(
                    "collection",
                    vec![concrete_type("string", vec![])],
                )],
            },
        )]),
    };

    let passthrough_library = NodeLibrary {
        id: "example.passthrough".into(),
        types: BTreeMap::from([(
            "passthrough".into(),
            NodeType {
                id: "passthrough".into(),
                type_params: vec![TypeParam {
                    id: "T".into(),
                    bounds: vec!["StringCollection".into()],
                }],
                inputs: BTreeMap::from([(
                    "collection".into(),
                    Port {
                        id: "collection".into(),
                        signal_type: SignalType::Var { param: "T".into() },
                        default_value: None,
                    },
                )]),
                outputs: BTreeMap::from([(
                    "collection".into(),
                    Port {
                        id: "collection".into(),
                        signal_type: SignalType::Var { param: "T".into() },
                        default_value: None,
                    },
                )]),
                implementation: None,
            },
        )]),
    };

    let registry = load_type_constraint_library(
        load_library(
            load_library(create_registry(), core_node_library()).unwrap(),
            passthrough_library,
        )
        .unwrap(),
        collection_constraint_library,
    )
    .unwrap();

    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![concrete_type(
                        "collection",
                        vec![concrete_type("string", vec![])],
                    )],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "p".into(),
                Node {
                    id: "p".into(),
                    node_type: "passthrough".into(),
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
                    node: "p".into(),
                    port: "collection".into(),
                },
            },
        )]),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}
