//! Graph resolve tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::core_node_library;
use imp_graph_resolve::{build_execution_program, flatten_graph, subgraph_key};
use imp_registry::{create_registry, load_graph_library, load_node_library, Registry};
use imp_core_types::{
    concrete_type, list_of, BoundaryBindings, Graph, GraphLibrary, Node, NodeDefinition, Port,
    PortReference,
};

fn passthrough_library() -> GraphLibrary {
    GraphLibrary {
        id: "example.graphs".into(),
        definitions: vec![NodeDefinition {
            id: "passthrough".into(),
            name: None,
            type_params: vec![],
            implementation: None,
            inputs: [(
                "rows".into(),
                Port {
                    id: "rows".into(),
                    signal_type: imp_core_types::concrete_type("collection", vec![]),
                    default_value: None,
                            variadic: false,
        },
            )]
            .into_iter()
            .collect(),
            outputs: [(
                "rows".into(),
                Port {
                    id: "rows".into(),
                    signal_type: imp_core_types::concrete_type("collection", vec![]),
                    default_value: None,
                            variadic: false,
        },
            )]
            .into_iter()
            .collect(),
            bindings: Some(BoundaryBindings {
                inputs: [("rows".into(), "in_rows".into())].into_iter().collect(),
                outputs: [("rows".into(), "out_rows".into())].into_iter().collect(),
            }),
            body: Some(Graph {
                nodes: [
                    (
                        "in_rows".into(),
                        Node {
                            id: "in_rows".into(),
                            node_type: "input".into(),
                            type_args: vec![],
                            inputs: Default::default(),
                        },
                    ),
                    (
                        "out_rows".into(),
                        Node {
                            id: "out_rows".into(),
                            node_type: "output".into(),
                            type_args: vec![],
                            inputs: Default::default(),
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                edges: [(
                    "e1".into(),
                    imp_core_types::Edge {
                        from: PortReference {
                            node: "in_rows".into(),
                            port: "value".into(),
                        },
                        to: PortReference {
                            node: "out_rows".into(),
                            port: "value".into(),
                        },
                    
                    order: None,
        },
                )]
                .into_iter()
                .collect(),
            }),
        }],
    }
}

fn build_registry() -> Registry {
    let registry = create_registry();
    let registry = load_node_library(registry, core_node_library()).unwrap();
    let registry = load_node_library(registry, collection_transforms_library()).unwrap();
    load_graph_library(registry, passthrough_library()).unwrap()
}

#[test]
fn flatten_graph_expands_composite_instance() {
    let registry = build_registry();
    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "comp".into(),
                Node {
                    id: "comp".into(),
                    node_type: "passthrough".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into_iter()
        .collect(),
        edges: [
            (
                "e1".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "comp".into(),
                        port: "rows".into(),
                    },
                
                order: None,
        },
            ),
            (
                "e2".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "comp".into(),
                        port: "rows".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                
                order: None,
        },
            ),
        ]
        .into_iter()
        .collect(),
    };

    let flat = flatten_graph(&graph, &registry).unwrap();
    assert!(!flat.nodes.contains_key("comp"));
    assert!(!flat.nodes.contains_key("comp::in_rows"));
    assert_eq!(
        flat.nodes.get("comp::out_rows").map(|n| n.node_type.as_str()),
        Some("output")
    );
    assert!(flat.edges.values().any(|edge| {
        edge.from.node == "in" && edge.to.node.starts_with("comp::")
    }));
    assert!(flat.edges.values().any(|edge| {
        edge.from.node.starts_with("comp::") && edge.to.node == "out"
    }));
}

#[test]
fn build_execution_program_deduplicates_shared_subgraphs() {
    let registry = build_registry();
    let graph = Graph {
        nodes: [
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "a".into(),
                Node {
                    id: "a".into(),
                    node_type: "passthrough".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "b".into(),
                Node {
                    id: "b".into(),
                    node_type: "passthrough".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into_iter()
        .collect(),
        edges: [
            (
                "e1".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "a".into(),
                        port: "rows".into(),
                    },
                
                order: None,
        },
            ),
            (
                "e2".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "a".into(),
                        port: "rows".into(),
                    },
                    to: PortReference {
                        node: "b".into(),
                        port: "rows".into(),
                    },
                
                order: None,
        },
            ),
            (
                "e3".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "b".into(),
                        port: "rows".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                
                order: None,
        },
            ),
        ]
        .into_iter()
        .collect(),
    };

    let program = build_execution_program(&graph, &registry).unwrap();
    assert_eq!(program.subgraphs.len(), 1);
    assert!(program.subgraphs.contains_key(&subgraph_key("passthrough", None)));
    assert_eq!(
        program.root.nodes.get("a").map(|n| n.node_type.as_str()),
        Some("passthrough")
    );
}

fn variadic_list_passthrough_library() -> GraphLibrary {
    GraphLibrary {
        id: "example.variadic".into(),
        definitions: vec![NodeDefinition {
            id: "list_passthrough".into(),
            name: None,
            type_params: vec![],
            implementation: None,
            inputs: [(
                "items".into(),
                Port {
                    id: "items".into(),
                    signal_type: concrete_type("collection", vec![]),
                    default_value: None,
                    variadic: true,
                },
            )]
            .into_iter()
            .collect(),
            outputs: [(
                "rows".into(),
                Port {
                    id: "rows".into(),
                    signal_type: list_of(concrete_type("collection", vec![])),
                    default_value: None,
                    variadic: false,
                },
            )]
            .into_iter()
            .collect(),
            bindings: Some(BoundaryBindings {
                inputs: [("items".into(), "in_items".into())].into_iter().collect(),
                outputs: [("rows".into(), "out_rows".into())].into_iter().collect(),
            }),
            body: Some(Graph {
                nodes: [
                    (
                        "in_items".into(),
                        Node {
                            id: "in_items".into(),
                            node_type: "input".into(),
                            type_args: vec![list_of(concrete_type("collection", vec![]))],
                            inputs: Default::default(),
                        },
                    ),
                    (
                        "out_rows".into(),
                        Node {
                            id: "out_rows".into(),
                            node_type: "output".into(),
                            type_args: vec![list_of(concrete_type("collection", vec![]))],
                            inputs: Default::default(),
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                edges: [(
                    "e1".into(),
                    imp_core_types::Edge {
                        from: PortReference {
                            node: "in_items".into(),
                            port: "value".into(),
                        },
                        to: PortReference {
                            node: "out_rows".into(),
                            port: "value".into(),
                        },
                        order: None,
                    },
                )]
                .into_iter()
                .collect(),
            }),
        }],
    }
}

#[test]
fn flatten_graph_inserts_pack_for_variadic_composite_input() {
    let registry = create_registry();
    let registry = load_node_library(registry, core_node_library()).unwrap();
    let registry = load_node_library(registry, collection_transforms_library()).unwrap();
    let registry = load_graph_library(registry, variadic_list_passthrough_library()).unwrap();

    let graph = Graph {
        nodes: [
            (
                "a".into(),
                Node {
                    id: "a".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "b".into(),
                Node {
                    id: "b".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "comp".into(),
                Node {
                    id: "comp".into(),
                    node_type: "list_passthrough".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: Default::default(),
                },
            ),
        ]
        .into_iter()
        .collect(),
        edges: [
            (
                "e1".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "a".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "comp".into(),
                        port: "items".into(),
                    },
                    order: Some(0),
                },
            ),
            (
                "e2".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "b".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "comp".into(),
                        port: "items".into(),
                    },
                    order: Some(1),
                },
            ),
            (
                "e3".into(),
                imp_core_types::Edge {
                    from: PortReference {
                        node: "comp".into(),
                        port: "rows".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]
        .into_iter()
        .collect(),
    };

    let flat = flatten_graph(&graph, &registry).unwrap();
    let pack = flat
        .nodes
        .values()
        .find(|n| n.node_type == "pack")
        .expect("expected pack node after flatten");
    assert_eq!(pack.node_type, "pack");
    let item_edges: Vec<_> = flat
        .edges
        .values()
        .filter(|e| e.to.node == pack.id && e.to.port == "items")
        .collect();
    assert_eq!(item_edges.len(), 2);
    assert!(flat.edges.values().any(|e| {
        e.from.node == pack.id && e.from.port == "list" && e.to.node.starts_with("comp::")
    }));
}
