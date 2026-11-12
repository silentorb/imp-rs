mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PortReference, PrimitiveValue};
use imp_execution::{
    execute_graph, ExecutionHost, ExecutionRow, ExecuteGraphOptions, ExecutionError,
};
use serde_json::json;
use std::collections::BTreeMap;

struct ErrorPathHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for ErrorPathHost {
    fn list_input_rows(&self) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Ok(self.rows.clone())
    }

    fn traverse(
        &self,
        _source_id: &str,
        _association: &str,
        _direction: u8,
        _edge_property: Option<&str>,
        _edge_equals: Option<&serde_json::Value>,
    ) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Ok(vec![])
    }
}

fn basic_in_out() -> (Node, Node) {
    (
        Node {
            id: "in".into(),
            node_type: "input".into(),
            type_args: vec![],
            inputs: BTreeMap::new(),
        },
        Node {
            id: "out".into(),
            node_type: "output".into(),
            type_args: vec![],
            inputs: BTreeMap::new(),
        },
    )
}

#[test]
fn limit_count_must_be_number() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
            (
                "limit".into(),
                Node {
                    id: "limit".into(),
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "count".into(),
                        PrimitiveValue::String("x".into()),
                    )]),
                },
            ),
            ("out".into(), output),
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
                    order: None,
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
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(err.to_string().contains("limit.count"));
}

#[test]
fn sort_column_wire_must_resolve_to_literal() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
            (
                "eq".into(),
                Node {
                    id: "eq".into(),
                    node_type: "equals".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "sort".into(),
                Node {
                    id: "sort".into(),
                    node_type: "sort".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            ("out".into(), output),
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
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_col".into(),
                Edge {
                    from: PortReference {
                        node: "eq".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "sort".into(),
                        port: "column".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(err.to_string().contains("sort.column"));
}

#[test]
fn contains_non_string_returns_false() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
            (
                "col".into(),
                Node {
                    id: "col".into(),
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("score".into()),
                    )]),
                },
            ),
            (
                "lit".into(),
                Node {
                    id: "lit".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::String("1".into()))]),
                },
            ),
            (
                "contains".into(),
                Node {
                    id: "contains".into(),
                    node_type: "contains".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
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
            ("out".into(), output),
        ]),
        edges: BTreeMap::from([
            (
                "e_col".into(),
                Edge {
                    from: PortReference {
                        node: "col".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "contains".into(),
                        port: "haystack".into(),
                    },
                    order: None,
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
                        node: "contains".into(),
                        port: "needle".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_pred".into(),
                Edge {
                    from: PortReference {
                        node: "contains".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                    order: None,
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
                    order: None,
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
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A").with_property("score", json!(99))],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert!(result.rows.is_empty());
}

#[test]
fn explicit_source_on_single_input_graph() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([("in".into(), input), ("out".into(), output)]),
        edges: BTreeMap::from([(
            "e1".into(),
            Edge {
                    from: PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                    to: PortReference {
                    node: "out".into(),
                    port: "value".into(),
                },
                    order: None,
                },
        )]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.source = Some(PortReference {
        node: "in".into(),
        port: "value".into(),
    });
    let result = execute_graph(&graph, options).unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn traverse_invalid_edge_property_type_errors() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
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
                        ("edge_property".into(), PrimitiveValue::Number(1.0)),
                        ("edge_equals".into(), PrimitiveValue::Null),
                    ]),
                },
            ),
            ("out".into(), output),
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
                    order: None,
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
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("S")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(err.to_string().contains("traverse.edge_property"));
}

#[test]
fn column_name_must_be_string() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
            (
                "col".into(),
                Node {
                    id: "col".into(),
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("name".into(), PrimitiveValue::Number(1.0))]),
                },
            ),
            (
                "lit".into(),
                Node {
                    id: "lit".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Number(1.0))]),
                },
            ),
            (
                "eq".into(),
                Node {
                    id: "eq".into(),
                    node_type: "equals".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
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
            ("out".into(), output),
        ]),
        edges: BTreeMap::from([
            (
                "e_col".into(),
                Edge {
                    from: PortReference {
                        node: "col".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "eq".into(),
                        port: "left".into(),
                    },
                    order: None,
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
                        node: "eq".into(),
                        port: "right".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_pred".into(),
                Edge {
                    from: PortReference {
                        node: "eq".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                    order: None,
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
                    order: None,
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
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(err.to_string().contains("column.name"));
}

#[test]
fn no_output_boundary_node_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([(
            "in".into(),
            Node {
                id: "in".into(),
                node_type: "input".into(),
                type_args: vec![],
                inputs: BTreeMap::new(),
            },
        )]),
        edges: BTreeMap::new(),
    };
    let host = ErrorPathHost { rows: vec![] };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::NoBoundaryNode(_)));
}

#[test]
fn explicit_non_output_sink_evaluates_upstream_port() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
            (
                "limit".into(),
                Node {
                    id: "limit".into(),
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(1.0))]),
                },
            ),
            ("out".into(), output),
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
                    order: None,
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
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A"), ExecutionRow::new("B")],
    };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.sink = Some(PortReference {
        node: "limit".into(),
        port: "collection".into(),
    });
    let result = execute_graph(&graph, options).unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn unknown_node_type_in_registry_errors() {
    let registry = test_registry();
    let (input, output) = basic_in_out();
    let graph = Graph {
        nodes: BTreeMap::from([
            ("in".into(), input),
            (
                "bad".into(),
                Node {
                    id: "bad".into(),
                    node_type: "totally_unknown".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            ("out".into(), output),
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
                        node: "bad".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "bad".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = ErrorPathHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(
        matches!(err, ExecutionError::Message(_))
            || matches!(err, ExecutionError::UnknownNodeType(_))
    );
}
