mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PrimitiveValue};
use imp_execution::{execute_graph, ExecutionHost, ExecutionRow, ExecuteGraphOptions, ExecutionError};
use serde_json::json;
use std::collections::BTreeMap;

struct PipelineHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for PipelineHost {
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

fn pipeline_graph() -> Graph {
    Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "col".into(),
                Node {
                    id: "col".into(),
                    title: None,
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("status".into()),
                    )]),
                },
            ),
            (
                "lit".into(),
                Node {
                    id: "lit".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "value".into(),
                        PrimitiveValue::String("active".into()),
                    )]),
                },
            ),
            (
                "eq".into(),
                Node {
                    id: "eq".into(),
                    title: None,
                    node_type: "equals".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "filter".into(),
                Node {
                    id: "filter".into(),
                    title: None,
                    node_type: "filter".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "sort".into(),
                Node {
                    id: "sort".into(),
                    title: None,
                    node_type: "sort".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([
                        (
                            "column".into(),
                            PrimitiveValue::String("title".into()),
                        ),
                        (
                            "direction".into(),
                            PrimitiveValue::String("asc".into()),
                        ),
                    ]),
                },
            ),
            (
                "limit".into(),
                Node {
                    id: "limit".into(),
                    title: None,
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(10.0))]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    title: None,
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
                "e_lit".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "lit".into(),
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
                "e_filter".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "filter".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_sort".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "limit".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "limit".into(),
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
    }
}

#[test]
fn pipeline_filter_sort_limit() {
    let registry = test_registry();
    let host = PipelineHost {
        rows: vec![
            ExecutionRow::new("1")
                .with_property("status", json!("active"))
                .with_property("title", json!("Zeta")),
            ExecutionRow::new("2")
                .with_property("status", json!("inactive"))
                .with_property("title", json!("Alpha")),
            ExecutionRow::new("3")
                .with_property("status", json!("active"))
                .with_property("title", json!("Beta")),
        ],
    };
    let result = execute_graph(
        &pipeline_graph(),
        ExecuteGraphOptions::new(&registry, &host),
    )
    .unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("3"));
    assert_eq!(result.rows[1].get("id").and_then(|v| v.as_str()), Some("1"));
}

#[test]
fn sort_direction_default_asc() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "sort".into(),
                Node {
                    id: "sort".into(),
                    title: None,
                    node_type: "sort".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "column".into(),
                        PrimitiveValue::String("name".into()),
                    )]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    title: None,
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
                        node: "sort".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "sort".into(),
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
    let host = PipelineHost {
        rows: vec![
            ExecutionRow::new("b").with_property("name", json!("Bravo")),
            ExecutionRow::new("a").with_property("name", json!("Alpha")),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("a"));
}

#[test]
fn group_sorts_by_column() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "group".into(),
                Node {
                    id: "group".into(),
                    title: None,
                    node_type: "group".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([
                        (
                            "column".into(),
                            PrimitiveValue::String("priority".into()),
                        ),
                        (
                            "direction".into(),
                            PrimitiveValue::String("desc".into()),
                        ),
                    ]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    title: None,
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
                        node: "group".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "group".into(),
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
    let host = PipelineHost {
        rows: vec![
            ExecutionRow::new("low").with_property("priority", json!(1)),
            ExecutionRow::new("high").with_property("priority", json!(3)),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("high"));
}

#[test]
fn project_columns() {
    let registry = test_registry();
    let graph = Graph {
        nodes: BTreeMap::from([
            (
                "in".into(),
                Node {
                    id: "in".into(),
                    title: None,
                    node_type: "input".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "project".into(),
                Node {
                    id: "project".into(),
                    title: None,
                    node_type: "project".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "columns".into(),
                        PrimitiveValue::String("title".into()),
                    )]),
                },
            ),
            (
                "out".into(),
                Node {
                    id: "out".into(),
                    title: None,
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
                        node: "project".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "project".into(),
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
    let host = PipelineHost {
        rows: vec![
            ExecutionRow::new("A")
                .with_property("title", json!("Alpha"))
                .with_property("extra", json!("drop")),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.columns, vec!["id", "title"]);
    assert!(result.rows[0].get("extra").is_none());
}

#[test]
fn moderate_size_filter_sort_limit() {
    let registry = test_registry();
    let mut rows = Vec::with_capacity(10_000);
    for i in 0..10_000 {
        let status = if i % 2 == 0 { "active" } else { "inactive" };
        rows.push(
            ExecutionRow::new(format!("row-{i}"))
                .with_property("status", json!(status))
                .with_property("title", json!(format!("T{i:05}"))),
        );
    }
    let host = PipelineHost { rows };
    let result = execute_graph(
        &pipeline_graph(),
        ExecuteGraphOptions::new(&registry, &host),
    )
    .unwrap();
    assert_eq!(result.rows.len(), 10);
}
