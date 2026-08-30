mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PortReference, PrimitiveValue};
use imp_execution::{
    execute_graph, ExecutionHost, ExecutionRow, ExecuteGraphOptions, ExecutionError,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

struct EdgeCaseHost {
    rows: Vec<ExecutionRow>,
    traverse: HashMap<(String, String, u8), Vec<ExecutionRow>>,
}

impl ExecutionHost for EdgeCaseHost {
    fn list_input_rows(&self) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Ok(self.rows.clone())
    }

    fn traverse(
        &self,
        source_id: &str,
        association: &str,
        direction: u8,
        _edge_property: Option<&str>,
        _edge_equals: Option<&serde_json::Value>,
    ) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Ok(self
            .traverse
            .get(&(source_id.to_string(), association.to_string(), direction))
            .cloned()
            .unwrap_or_default())
    }
}

#[test]
fn less_than_and_not_equals_predicates() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "col".into(),
                Node {
                    id: "col".into(),
                    node_type: "column".into(),
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("n".into()),
                    )]),
                },
            ),
            (
                "lit".into(),
                Node {
                    id: "lit".into(),
                    node_type: "literal".into(),
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Number(3.0))]),
                },
            ),
            (
                "lt".into(),
                Node {
                    id: "lt".into(),
                    node_type: "less_than".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "ne".into(),
                Node {
                    id: "ne".into(),
                    node_type: "not_equals".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "or".into(),
                Node {
                    id: "or".into(),
                    node_type: "or".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    node_type: "filter".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e_col_lt".into(),
                Edge {
                    from: PortReference {
                        node: "col".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "lt".into(),
                        port: "left".into(),
                    },
                },
            ),
            (
                "e_lit_lt".into(),
                Edge {
                    from: PortReference {
                        node: "lit".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "lt".into(),
                        port: "right".into(),
                    },
                },
            ),
            (
                "e_col_ne".into(),
                Edge {
                    from: PortReference {
                        node: "col".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "ne".into(),
                        port: "left".into(),
                    },
                },
            ),
            (
                "e_lit_ne".into(),
                Edge {
                    from: PortReference {
                        node: "lit".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "ne".into(),
                        port: "right".into(),
                    },
                },
            ),
            (
                "e_or_l".into(),
                Edge {
                    from: PortReference {
                        node: "lt".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "or".into(),
                        port: "left".into(),
                    },
                },
            ),
            (
                "e_or_r".into(),
                Edge {
                    from: PortReference {
                        node: "ne".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "or".into(),
                        port: "right".into(),
                    },
                },
            ),
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
    let host = EdgeCaseHost {
        rows: vec![
            ExecutionRow::new("small").with_property("n", json!(2)),
            ExecutionRow::new("equal").with_property("n", json!(3)),
            ExecutionRow::new("large").with_property("n", json!(5)),
        ],
        traverse: HashMap::new(),
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    let ids: Vec<_> = result
        .rows
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["small", "large"]);
}

#[test]
fn sort_nulls_last() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "sort".into(),
                Node {
                    id: "sort".into(),
                    node_type: "sort".into(),
                    inputs: BTreeMap::from([
                        (
                            "column".into(),
                            PrimitiveValue::String("name".into()),
                        ),
                        (
                            "direction".into(),
                            PrimitiveValue::String("asc".into()),
                        ),
                    ]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
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
                        node: "sort".into(),
                        port: "collection".into(),
                    },
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
                },
            ),
        ]),
    };
    let host = EdgeCaseHost {
        rows: vec![
            ExecutionRow::new("null-row"),
            ExecutionRow::new("named").with_property("name", json!("Amy")),
        ],
        traverse: HashMap::new(),
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("named"));
    assert_eq!(
        result.rows[1].get("id").and_then(|v| v.as_str()),
        Some("null-row")
    );
}

#[test]
fn project_properties_prefix_and_is_archived() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "project".into(),
                Node {
                    id: "project".into(),
                    node_type: "project".into(),
                    inputs: BTreeMap::from([(
                        "columns".into(),
                        PrimitiveValue::String("properties.title,is_archived".into()),
                    )]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
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
                        node: "project".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "project".into(),
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
    let mut row = ExecutionRow::new("A").with_property("title", json!("Hello"));
    row.is_archived = Some(true);
    let host = EdgeCaseHost {
        rows: vec![row],
        traverse: HashMap::new(),
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows[0].get("title").and_then(|v| v.as_str()), Some("Hello"));
}

#[test]
fn traverse_invalid_direction_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "hop".into(),
                Node {
                    id: "hop".into(),
                    node_type: "traverse".into(),
                    inputs: BTreeMap::from([
                        (
                            "association".into(),
                            PrimitiveValue::String("knows".into()),
                        ),
                        ("direction".into(), PrimitiveValue::Number(2.0)),
                    ]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
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
    let host = EdgeCaseHost {
        rows: vec![ExecutionRow::new("S")],
        traverse: HashMap::new(),
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::Message(_)));
}

#[test]
fn multiple_boundary_nodes_error() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in1".into(),
                Node {
                    id: "in1".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "in2".into(),
                Node {
                    id: "in2".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::new(),
    };
    let host = EdgeCaseHost {
        rows: vec![],
        traverse: HashMap::new(),
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::MultipleBoundaryNodes { .. }));
}

#[test]
fn unwired_explicit_sink_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "limit".into(),
                Node {
                    id: "limit".into(),
                    node_type: "limit".into(),
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(1.0))]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
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
    let host = EdgeCaseHost {
        rows: vec![ExecutionRow::new("A"), ExecutionRow::new("B")],
        traverse: HashMap::new(),
    };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.sink = Some(PortReference {
        node: "missing".into(),
        port: "collection".into(),
    });
    let err = execute_graph(&graph, options).unwrap_err();
    assert!(matches!(err, ExecutionError::UnknownNode(_)));
}

#[test]
fn explicit_sink_without_incoming_wire_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "limit".into(),
                Node {
                    id: "limit".into(),
                    node_type: "limit".into(),
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(1.0))]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([(
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
        )]),
    };
    let host = EdgeCaseHost {
        rows: vec![ExecutionRow::new("A")],
        traverse: HashMap::new(),
    };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.sink = Some(PortReference {
        node: "limit".into(),
        port: "collection".into(),
    });
    let err = execute_graph(&graph, options).unwrap_err();
    assert!(matches!(err, ExecutionError::UnwiredSink { .. }));
}

#[test]
fn sort_desc_and_traverse_direction_one() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "param".into(),
                Node {
                    id: "param".into(),
                    node_type: "parameter".into(),
                    inputs: BTreeMap::from([
                        (
                            "label".into(),
                            PrimitiveValue::String("Col".into()),
                        ),
                        (
                            "value".into(),
                            PrimitiveValue::String("name".into()),
                        ),
                    ]),
                },
            ),
            (
                "sort".into(),
                Node {
                    id: "sort".into(),
                    node_type: "sort".into(),
                    inputs: BTreeMap::from([(
                        "direction".into(),
                        PrimitiveValue::String("desc".into()),
                    )]),
                },
            ),
            (
                "hop".into(),
                Node {
                    id: "hop".into(),
                    node_type: "traverse".into(),
                    inputs: BTreeMap::from([
                        (
                            "association".into(),
                            PrimitiveValue::String("knows".into()),
                        ),
                        ("direction".into(), PrimitiveValue::Number(1.0)),
                    ]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([
            (
                "e_param".into(),
                Edge {
                    from: PortReference {
                        node: "param".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "sort".into(),
                        port: "column".into(),
                    },
                },
            ),
            (
                "e_in_sort".into(),
                Edge {
                    from: PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: PortReference {
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_sort_hop".into(),
                Edge {
                    from: PortReference {
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                    to: PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_out".into(),
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
    let host = EdgeCaseHost {
        rows: vec![
            ExecutionRow::new("a").with_property("name", json!("Alpha")),
            ExecutionRow::new("b").with_property("name", json!("Zulu")),
        ],
        traverse: HashMap::from([(("a".into(), "knows".into(), 1), vec![ExecutionRow::new("T")])]),
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("T"));
}

#[test]
fn filter_predicate_must_be_wired() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    node_type: "filter".into(),
                    inputs: BTreeMap::from([(
                        "predicate".into(),
                        PrimitiveValue::Bool(true),
                    )]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
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
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e2".into(),
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
    let host = EdgeCaseHost {
        rows: vec![ExecutionRow::new("A")],
        traverse: HashMap::new(),
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::Message(_)));
}

#[test]
fn project_empty_columns_errors() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    node_type: "input".into(),
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "project".into(),
                Node {
                    id: "project".into(),
                    node_type: "project".into(),
                    inputs: BTreeMap::from([(
                        "columns".into(),
                        PrimitiveValue::String("  ,  ".into()),
                    )]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
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
                        node: "project".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: PortReference {
                        node: "project".into(),
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
    let host = EdgeCaseHost {
        rows: vec![ExecutionRow::new("A")],
        traverse: HashMap::new(),
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::Message(_)));
}
