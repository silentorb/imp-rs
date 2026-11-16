mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PrimitiveValue};
use imp_execution::{
    execute_graph, ExecutionHost, ExecutionRow, ExecuteGraphOptions, ExecutionError,
};
use serde_json::json;
use std::collections::BTreeMap;

struct PredicateHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for PredicateHost {
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
fn contains_predicate_case_insensitive() {
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
                "col".into(),
                Node {
                    id: "col".into(),
                    title: None,
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("title".into()),
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
                        PrimitiveValue::String("beta".into()),
                    )]),
                },
            ),
            (
                "contains".into(),
                Node {
                    id: "contains".into(),
                    title: None,
                    node_type: "contains".into(),
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
                        node: "contains".into(),
                        port: "haystack".into(),
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
                        node: "contains".into(),
                        port: "needle".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_pred".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "contains".into(),
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
    let host = PredicateHost {
        rows: vec![
            ExecutionRow::new("A").with_property("title", json!("Alpha Beta")),
            ExecutionRow::new("B").with_property("title", json!("Gamma")),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("A"));
}

#[test]
fn not_predicate_excludes_matching_rows() {
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
                "col".into(),
                Node {
                    id: "col".into(),
                    title: None,
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("active".into()),
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
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Bool(true))]),
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
                "not".into(),
                Node {
                    id: "not".into(),
                    title: None,
                    node_type: "not".into(),
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
                "e_eq".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "eq".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "not".into(),
                        port: "value".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_pred".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "not".into(),
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
    let host = PredicateHost {
        rows: vec![
            ExecutionRow::new("A").with_property("active", json!(true)),
            ExecutionRow::new("B").with_property("active", json!(false)),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("B"));
}

#[test]
fn and_or_predicates() {
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
                "c1".into(),
                Node {
                    id: "c1".into(),
                    title: None,
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("a".into()),
                    )]),
                },
            ),
            (
                "l1".into(),
                Node {
                    id: "l1".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Number(1.0))]),
                },
            ),
            (
                "eq1".into(),
                Node {
                    id: "eq1".into(),
                    title: None,
                    node_type: "equals".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "c2".into(),
                Node {
                    id: "c2".into(),
                    title: None,
                    node_type: "column".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "name".into(),
                        PrimitiveValue::String("b".into()),
                    )]),
                },
            ),
            (
                "l2".into(),
                Node {
                    id: "l2".into(),
                    title: None,
                    node_type: "literal".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("value".into(), PrimitiveValue::Number(2.0))]),
                },
            ),
            (
                "eq2".into(),
                Node {
                    id: "eq2".into(),
                    title: None,
                    node_type: "equals".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
                },
            ),
            (
                "and".into(),
                Node {
                    id: "and".into(),
                    title: None,
                    node_type: "and".into(),
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
                "ec1".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "c1".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "eq1".into(),
                        port: "left".into(),
                    },
                    order: None,
                },
            ),
            (
                "el1".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "l1".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "eq1".into(),
                        port: "right".into(),
                    },
                    order: None,
                },
            ),
            (
                "ec2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "c2".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "eq2".into(),
                        port: "left".into(),
                    },
                    order: None,
                },
            ),
            (
                "el2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "l2".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "eq2".into(),
                        port: "right".into(),
                    },
                    order: None,
                },
            ),
            (
                "eand_l".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "eq1".into(),
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
                "eand_r".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "eq2".into(),
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
                "e_pred".into(),
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
    let host = PredicateHost {
        rows: vec![
            ExecutionRow::new("match")
                .with_property("a", json!(1))
                .with_property("b", json!(2)),
            ExecutionRow::new("miss")
                .with_property("a", json!(1))
                .with_property("b", json!(0)),
        ],
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(
        result.rows[0].get("id").and_then(|v| v.as_str()),
        Some("match")
    );
}

#[test]
fn cycle_detection() {
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
                "limit1".into(),
                Node {
                    id: "limit1".into(),
                    title: None,
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(5.0))]),
                },
            ),
            (
                "limit2".into(),
                Node {
                    id: "limit2".into(),
                    title: None,
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([("count".into(), PrimitiveValue::Number(5.0))]),
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
                "e_cycle1".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "limit2".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "limit1".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_cycle2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "limit1".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "limit2".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "limit1".into(),
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
    let host = PredicateHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::CycleDetected(_)));
}

#[test]
fn unsatisfied_limit_count() {
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
                "limit".into(),
                Node {
                    id: "limit".into(),
                    title: None,
                    node_type: "limit".into(),
                    type_args: vec![],
                    inputs: BTreeMap::new(),
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
                        node: "limit".into(),
                        port: "collection".into(),
                    },
                    order: None,
                },
            ),
            (
                "e2".into(),
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
    };
    let host = PredicateHost { rows: vec![] };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::UnsatisfiedInput { .. }));
}

#[test]
fn search_requires_text_search() {
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
                "search".into(),
                Node {
                    id: "search".into(),
                    title: None,
                    node_type: "search".into(),
                    type_args: vec![],
                    inputs: BTreeMap::from([(
                        "query".into(),
                        PrimitiveValue::String("foo".into()),
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
    let host = PredicateHost {
        rows: vec![ExecutionRow::new("A")],
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::MissingTextSearch));
}
