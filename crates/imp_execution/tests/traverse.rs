mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node, PrimitiveValue};
use imp_execution::{
    execute_graph, ExecutionHost, ExecutionRow, ExecuteGraphOptions, ExecutionError,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

type TraverseKey = (String, String, u8);

struct TraverseHost {
    rows: Vec<ExecutionRow>,
    edges: HashMap<TraverseKey, Vec<ExecutionRow>>,
}

impl TraverseHost {
    fn key(source_id: &str, association: &str, direction: u8) -> TraverseKey {
        (source_id.to_string(), association.to_string(), direction)
    }
}

impl ExecutionHost for TraverseHost {
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
            .edges
            .get(&Self::key(source_id, association, direction))
            .cloned()
            .unwrap_or_default())
    }
}

fn traverse_graph() -> Graph {
    Graph {
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
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e2".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    }
}

#[test]
fn traverse_follows_host_edges_and_dedupes() {
    let registry = test_registry();
    let target = ExecutionRow::new("T1").with_property("title", json!("Target"));
    let host = TraverseHost {
        rows: vec![
            ExecutionRow::new("S1"),
            ExecutionRow::new("S2"),
        ],
        edges: HashMap::from([
            (
                TraverseHost::key("S1", "knows", 0),
                vec![target.clone()],
            ),
            (
                TraverseHost::key("S2", "knows", 0),
                vec![target.clone(), ExecutionRow::new("T2")],
            ),
        ]),
    };
    let result = execute_graph(
        &traverse_graph(),
        ExecuteGraphOptions::new(&registry, &host),
    )
    .unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<_> = result
        .rows
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
        .collect();
    assert!(ids.contains(&"T1"));
    assert!(ids.contains(&"T2"));
}

#[test]
fn except_anti_membership() {
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
                    ]),
                },
            ),
            (
                "except".into(),
                Node {
                    id: "except".into(),
                    node_type: "except".into(),
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
                "e_keep".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "except".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_hop_in".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_excl".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "hop".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "except".into(),
                        port: "exclude".into(),
                    },
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "except".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = TraverseHost {
        rows: vec![
            ExecutionRow::new("A"),
            ExecutionRow::new("B"),
        ],
        edges: HashMap::from([(
            TraverseHost::key("A", "knows", 0),
            vec![ExecutionRow::new("B"), ExecutionRow::new("C")],
        )]),
    };
    let result = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap();
    let ids: Vec<_> = result
        .rows
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(ids, vec!["A"]);
}

#[test]
fn except_exclude_unwired_errors() {
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
                "except".into(),
                Node {
                    id: "except".into(),
                    node_type: "except".into(),
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
                "e_keep".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "in".into(),
                        port: "value".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "except".into(),
                        port: "collection".into(),
                    },
                },
            ),
            (
                "e_out".into(),
                Edge {
                    from: imp_core_types::PortReference {
                        node: "except".into(),
                        port: "collection".into(),
                    },
                    to: imp_core_types::PortReference {
                        node: "out".into(),
                        port: "value".into(),
                    },
                },
            ),
        ]),
    };
    let host = TraverseHost {
        rows: vec![],
        edges: HashMap::new(),
    };
    let err = execute_graph(&graph, ExecuteGraphOptions::new(&registry, &host)).unwrap_err();
    assert!(matches!(err, ExecutionError::UnsatisfiedInput { .. }));
}
