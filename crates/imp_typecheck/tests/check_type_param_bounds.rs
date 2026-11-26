//! Type-parameter bound checking tests.

use imp_core_types::{
    concrete_type, core_node_library, Edge, Graph, Node, NodeLibrary, NodeType, Port,
    PortReference, Type, TypeDefinition, TypeParam,
};
use imp_registry::{create_registry, load_library};
use imp_typecheck::check_graph;
use std::collections::BTreeMap;

/// `Scalar = string | number`.
fn scalar_type() -> TypeDefinition {
    TypeDefinition {
        id: "Scalar".into(),
        params: vec![],
        ty: imp_core_types::union_of(vec![
            concrete_type("string", vec![]),
            concrete_type("number", vec![]),
        ]),
    }
}

fn identity_library() -> NodeLibrary {
    NodeLibrary {
        types: vec![scalar_type()],
        id: "example.identity".into(),
        definitions: vec![NodeType {
            id: "identity".into(),
            name: None,
            type_params: vec![TypeParam {
                id: "T".into(),
                bounds: vec![concrete_type("Scalar", vec![])],
            }],
            body: None,
            bindings: None,
            implementation: None,
            inputs: BTreeMap::from([(
                "value".into(),
                Port {
                    id: "value".into(),
                    ty: Type::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
            outputs: BTreeMap::from([(
                "value".into(),
                Port {
                    id: "value".into(),
                    ty: Type::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
        }],
    }
}

fn registry_with_bounds() -> imp_registry::Registry {
    load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        identity_library(),
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![concrete_type("number", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "id".into(),
                Node {
                    id: "id".into(),
                    title: None,
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
            order: None,
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![concrete_type("boolean", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "id".into(),
                Node {
                    id: "id".into(),
                    title: None,
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
            order: None,
            },
        )]),
    };

    let errors = check_graph(&graph, &registry);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("does not satisfy bound string | number"))
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![concrete_type("any", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "id".into(),
                Node {
                    id: "id".into(),
                    title: None,
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
            order: None,
            },
        )]),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}

#[test]
fn check_graph_rejects_args_for_an_undefined_bound_name() {
    let bad_library = NodeLibrary {
        types: Vec::new(),
        id: "example.bad".into(),
        definitions: vec![NodeType {
            id: "bad".into(),
            name: None,
            type_params: vec![TypeParam {
                id: "T".into(),
                bounds: vec![concrete_type("Missing", vec![])],
            }],
            body: None,
            bindings: None,
            implementation: None,
            inputs: BTreeMap::from([(
                "value".into(),
                Port {
                    id: "value".into(),
                    ty: Type::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
            outputs: BTreeMap::from([(
                "value".into(),
                Port {
                    id: "value".into(),
                    ty: Type::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
        }],
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
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![concrete_type("number", vec![])],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "bad".into(),
                Node {
                    id: "bad".into(),
                    title: None,
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
            order: None,
            },
        )]),
    };

    let errors = check_graph(&graph, &registry);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("does not satisfy bound Missing"))
    );
}

#[test]
fn check_graph_matches_parametric_bound_types() {
    let passthrough_library = NodeLibrary {
        types: vec![TypeDefinition {
            id: "StringCollection".into(),
            params: vec![],
            ty: concrete_type("collection", vec![concrete_type("string", vec![])]),
        }],
        id: "example.passthrough".into(),
        definitions: vec![NodeType {
            id: "passthrough".into(),
            name: None,
            type_params: vec![TypeParam {
                id: "T".into(),
                bounds: vec![concrete_type("StringCollection", vec![])],
            }],
            body: None,
            bindings: None,
            implementation: None,
            inputs: BTreeMap::from([(
                "collection".into(),
                Port {
                    id: "collection".into(),
                    ty: Type::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
            outputs: BTreeMap::from([(
                "collection".into(),
                Port {
                    id: "collection".into(),
                    ty: Type::Var { param: "T".into() },
                    default_value: None,
                variadic: false,
                },
            )]),
        }],
    };

    let registry = load_library(
        load_library(create_registry(), core_node_library()).unwrap(),
        passthrough_library,
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
                    title: None,
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
            order: None,
            },
        )]),
    };

    assert!(check_graph(&graph, &registry).is_empty());
}
