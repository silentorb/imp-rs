//! imp-typecheck integration tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::{
    concrete_type, core_node_library, Edge, Graph, Node, PortReference, PrimitiveValue,
};
use imp_registry::{create_registry, load_library};
use imp_typecheck::check_graph;

#[test]
fn accepts_filter_wired_to_input_boundary() {
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

    assert!(check_graph(&graph, &registry).is_empty());
}
