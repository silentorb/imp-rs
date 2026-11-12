mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PrimitiveValue};
use imp_execution::{
    execute_graph, ExecutionCapabilities, ExecutionHost, ExecutionRow, ExecuteGraphOptions,
    ExecutionError,
};
use serde_json::json;
use std::collections::BTreeMap;

struct OffsetHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for OffsetHost {
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

#[test]
fn offset_skips_rows() {
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
                "offset".into(),
                Node {
                    id: "offset".into(),
                    node_type: "offset".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(1.0))]),
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
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "offset".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "offset".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost {
        rows: vec![
            ExecutionRow::new("first"),
            ExecutionRow::new("second"),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("second"));
}

#[test]
fn parameter_node_in_filter_predicate() {
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
                "col".into(),
                Node {
                    id: "col".into(),
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("status".into()),
                    )]),
                },
            ),
            (
                "param".into(),
                Node {
                    id: "param".into(),
                    node_type: "parameter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([
                        (
                            "label".into(),
                            PrimitiveValue::String("Status".into()),
                        ),
                        (
                            "value".into(),
                            PrimitiveValue::String("active".into()),
                        ),
                    ]),
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
                "e_col".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "col".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "eq".into(),
                        port: "left".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_param".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "param".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "eq".into(),
                        port: "right".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_pred".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "eq".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_in".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost {
        rows: vec![
            ExecutionRow::new("A").with_property("status", json!("active")),
            ExecutionRow::new("B").with_property("status", json!("inactive")),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("A"));
}

#[test]
fn comparison_predicates() {
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
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Number(5.0))]),
                },
            ),
            (
                "gt".into(),
                Node {
                    id: "gt".into(),
                    node_type: "greater_than".into(),
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
                "e_col".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "col".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "gt".into(),
                        port: "left".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_lit".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "lit".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "gt".into(),
                        port: "right".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_pred".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "gt".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_in".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost {
        rows: vec![
            ExecutionRow::new("high").with_property("score", json!(10)),
            ExecutionRow::new("low").with_property("score", json!(2)),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("high"));
}

struct SearchHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for SearchHost {
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

    fn text_search(
        &self,
        rows: &[ExecutionRow],
        query: &str,
    ) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Ok(rows
            .iter()
            .filter(|r| {
                r.properties
                    .get("title")
                    .and_then(|v| v.as_str())
                    .is_some_and(|t| t.contains(query))
            })
            .cloned()
            .collect())
    }
}

#[test]
fn search_delegates_to_host_text_search() {
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
                "search".into(),
                Node {
                    id: "search".into(),
                    node_type: "search".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "query".into(),
                        PrimitiveValue::String("Beta".into()),
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
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "search".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "search".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = SearchHost {
        rows: vec![
            ExecutionRow::new("A").with_property("title", json!("Alpha")),
            ExecutionRow::new("B").with_property("title", json!("Beta")),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("B"));
}

#[test]
fn rejects_missing_read_capability() {
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
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([(
            "e1".into(),
            Edge {
                    from: imp_core_types::PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                    to: imp_core_types::PortReference {
                    node: "out".into(),
                    port: "value".into(),
                },
                    order: None,
                },
        )]),
    };
    let host = OffsetHost { rows: vec![] };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.capabilities = ExecutionCapabilities {
        read: false,
        write: false,
    };
    let err = execute_graph(&graph, options).unwrap_err();
    assert!(matches!(err, ExecutionError::ReadCapabilityRequired));
}

#[test]
fn duplicate_edge_target_errors() {
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
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost { rows: vec![] };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::DuplicateEdgeTarget { .. }));
}

#[test]
fn unsupported_node_type_errors() {
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
                "weird".into(),
                Node {
                    id: "weird".into(),
                    node_type: "no_such_type".into(),
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
                "e1".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "weird".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "weird".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::Message(_)));
}

#[test]
fn empty_result_has_id_column() {
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
                "out".into(),
                Node {
                    id: "out".into(),
                    node_type: "output".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
        ]),
        edges: BTreeMap::from([(
            "e1".into(),
            Edge {
                    from: imp_core_types::PortReference {
                    node: "in".into(),
                    port: "value".into(),
                },
                    to: imp_core_types::PortReference {
                    node: "out".into(),
                    port: "value".into(),
                },
                    order: None,
                },
        )]),
    };
    let host = OffsetHost { rows: vec![] };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.columns, vec!["id"]);
    assert!(result.rows.is_empty());
}

#[test]
fn pack_concatenates_ordered_collection_wires() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in_a".into(),
                Node {
                    id: "in_a".into(),
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "pack".into(),
                Node {
                    id: "pack".into(),
                    node_type: "pack".into(),
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
                "e1".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in_a".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "pack".into(),
                        port: "items".into(),
                    },
                    order: Some(0),
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "pack".into(),
                        port: "list".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost {
        rows: vec![ExecutionRow::new("A"), ExecutionRow::new("B")],
    };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.source = Some(imp_core_types::PortReference {
        node: "in_a".into(),
        port: "value".into(),
    });
    let result = execute_graph(&graph, options).unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("A"));
    assert_eq!(result.rows[1].get("id").and_then(|v| v.as_str()), Some("B"));
}

#[test]
fn nary_and_folds_operands_in_edge_order() {
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
                "t1".into(),
                Node {
                    id: "t1".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Bool(true))]),
                },
            ),
            (
                "t2".into(),
                Node {
                    id: "t2".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Bool(true))]),
                },
            ),
            (
                "f".into(),
                Node {
                    id: "f".into(),
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Bool(false))]),
                },
            ),
            (
                "and".into(),
                Node {
                    id: "and".into(),
                    node_type: "and".into(),
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
                "e_in".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "ea".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "t1".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "and".into(),
                        port: "operands".into(),
                    },
                    order: Some(0),
                },
            ),
            (
                "eb".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "f".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "and".into(),
                        port: "operands".into(),
                    },
                    order: Some(1),
                },
            ),
            (
                "ec".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "t2".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "and".into(),
                        port: "operands".into(),
                    },
                    order: Some(2),
                },
            ),
            (
                "ep".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "and".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "predicate".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
        ]),
    };
    let host = OffsetHost {
        rows: vec![ExecutionRow::new("keep")],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert!(result.rows.is_empty());
}
