//! Graph-backed definition validation tests.

use imp_core_types::{
    concrete_type, core_node_library, BoundaryBindings, Graph, GraphLibrary, Node, NodeDefinition,
    Port, PortReference,
};
use imp_registry::{create_registry, load_graph_library, load_node_library};
use imp_typecheck::{check_definition, check_graph_definitions};

fn passthrough_definition() -> NodeDefinition {
    NodeDefinition {
        id: "passthrough".into(),
        name: Some("Passthrough".into()),
        type_params: vec![],
        implementation: None,
        inputs: [(
            "rows".into(),
            Port {
                id: "rows".into(),
                signal_type: concrete_type("collection", vec![]),
                default_value: None,
            },
        )]
        .into_iter()
        .collect(),
        outputs: [(
            "rows".into(),
            Port {
                id: "rows".into(),
                signal_type: concrete_type("collection", vec![]),
                default_value: None,
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
                        type_args: vec![concrete_type("collection", vec![])],
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
                },
            )]
            .into_iter()
            .collect(),
        }),
    }
}

#[test]
fn check_definition_validates_graph_backed_library_entry() {
    let registry = load_graph_library(
        load_node_library(create_registry(), core_node_library()).unwrap(),
        GraphLibrary {
            id: "example.graphs".into(),
            definitions: vec![passthrough_definition()],
        },
    )
    .unwrap();

    let definition = registry.definitions.get("passthrough").unwrap();
    assert!(check_definition(definition, &registry).is_empty());
    assert!(check_graph_definitions(&registry).is_empty());
}

#[test]
fn check_graph_definitions_reports_definition_cycle() {
    let mut a = passthrough_definition();
    a.id = "a".into();
    let mut b = passthrough_definition();
    b.id = "b".into();
    b.body.as_mut().unwrap().nodes.insert(
        "comp".into(),
        Node {
            id: "comp".into(),
            node_type: "a".into(),
            type_args: vec![],
            inputs: Default::default(),
        },
    );
    a.body.as_mut().unwrap().nodes.insert(
        "comp".into(),
        Node {
            id: "comp".into(),
            node_type: "b".into(),
            type_args: vec![],
            inputs: Default::default(),
        },
    );

    let registry = load_graph_library(
        load_node_library(create_registry(), core_node_library()).unwrap(),
        GraphLibrary {
            id: "cycle.lib".into(),
            definitions: vec![a, b],
        },
    )
    .unwrap();

    let errors = check_graph_definitions(&registry);
    assert!(errors.iter().any(|e| e.message.contains("definition-reference cycle")));
}
