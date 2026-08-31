mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PortReference, PrimitiveValue};
use imp_execution::{
    execute_graph, ExecutionHost, ExecutionRow, ExecuteGraphOptions, ExecutionError,
};
use serde_json::json;
use std::collections::BTreeMap;

struct ResolveHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for ResolveHost {
    fn list_input_rows(&self) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Ok(self.rows.clone())
    }

    fn traverse(
        &self,
        _source_id: &str,
        _association: &str,
        _direction: u8,
        edge_property: Option<&str>,
        edge_equals: Option<&serde_json::Value>,
    ) -> Result<Vec<ExecutionRow>, ExecutionError> {
        if edge_property == Some("priority") {
            if edge_equals == Some(&json!("high")) {
                return Ok(vec![ExecutionRow::new("T-high")]);
            }
            return Ok(vec![]);
        }
        Ok(vec![])
    }
}

#[test]
fn no_input_boundary_node_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([(
            "out".into(),
            Node {
                id: "out".into(),
                node_type: "output".into(),
                type_args: vec![],
                inputs: BTreeMap::new(),
            },
        )]),
        edges: BTreeMap::new(),
    };
    let host = ResolveHost { rows: vec![] };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::NoBoundaryNode(_)));
}

#[test]
fn unexpected_input_node_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "in2".into(),
                Node {
                    id: "in2".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "lit".into(),
                Node {
                    id: "lit".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Bool(true))]),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e_in2".into(),
                Edge {
                    from: PortReference {
                        node: "in2".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_lit".into(),
                Edge {
                    from: PortReference {
                        node: "lit".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = ResolveHost { rows: vec![] };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.source = Some(PortReference {
        node: "in".into(),
        port: "value".into(),
    });
    let err = execute_graph(&graph, options).unwrap_err();
    assert!(matches!(err, ExecutionError::Message(_)));
}

#[test]
fn traverse_with_edge_property_filter() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "hop".into(),
                Node {
                    id: "hop".into(),
                    node_type: "traverse".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([
                        (
                            "association".into(),
                            PrimitiveValue::String("knows".into()),
                        ),
                        ("direction".into(), PrimitiveValue::Number(0.0)),
                        (
                            "edge_property".into(),
                            PrimitiveValue::String("priority".into()),
                        ),
                        (
                            "edge_equals".into(),
                            PrimitiveValue::String("high".into()),
                        ),
                    ]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e1".into(),
                Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = ResolveHost {
        rows: vec![ExecutionRow::new("S1")],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(
        result.rows[0].get("id").and_then(|v| v.as_str()),
        Some("T-high")
    );
}

#[test]
fn and_requires_wired_inputs() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "and".into(),
                Node {
                    id: "and".into(),
                    node_type: "and".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "left".into(),
                        PrimitiveValue::Bool(true),
                    )]),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e_pred".into(),
                Edge {
                    from: PortReference {
                        node: "and".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e_in".into(),
                Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = ResolveHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(
        matches!(err, ExecutionError::Message(_))
            || matches!(err, ExecutionError::UnsatisfiedInput { .. })
    );
}

#[test]
fn unknown_port_on_node_type_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "limit".into(),
                Node {
                    id: "limit".into(),
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "not_a_port".into(),
                        PrimitiveValue::Number(1.0),
                    )]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e1".into(),
                Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "limit".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "limit".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = ResolveHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::UnsatisfiedInput { .. }));
}

#[test]
fn or_requires_wired_inputs() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "or".into(),
                Node {
                    id: "or".into(),
                    node_type: "or".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "left".into(),
                        PrimitiveValue::Bool(true),
                    )]),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e_pred".into(),
                Edge {
                    from: PortReference {
                        node: "or".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e_in".into(),
                Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = ResolveHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(err.to_string().contains("or"));
}

#[test]
fn not_requires_wired_input() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "not".into(),
                Node {
                    id: "not".into(),
                    node_type: "not".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "value".into(),
                        PrimitiveValue::Bool(false),
                    )]),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e_pred".into(),
                Edge {
                    from: PortReference {
                        node: "not".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                },
            ),
            (
                "e_in".into(),
                Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = ResolveHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(err.to_string().contains("not"));
}
