mod common;

use common::test_registry;
use imp_core_types::{Edge, Graph, Node};
use imp_execution::{
    execute_graph, ExecutionCapabilities, ExecutionHost, ExecutionRow, ExecuteGraphOptions,
    ExecutionError,
};
use serde_json::json;
use std::collections::BTreeMap;

struct PassthroughHost {
    rows: Vec<ExecutionRow>,
}

impl ExecutionHost for PassthroughHost {
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

fn passthrough_graph() -> Graph {
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
    }
}

#[test]
fn executes_input_to_output_passthrough() {
    let registry = test_registry();
    let host = PassthroughHost {
        rows: vec![
            ExecutionRow::new("A").with_property("title", json!("Alpha")),
            ExecutionRow::new("B").with_property("title", json!("Beta")),
        ],
    };
    let result = execute_graph(
        &passthrough_graph(),
        ExecuteGraphOptions::new(&registry, &host),
    )
    .unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].get("id").and_then(|v| v.as_str()), Some("A"));
}

#[test]
fn rejects_write_capability_grant() {
    let registry = test_registry();
    let host = PassthroughHost { rows: vec![] };
    let mut options = ExecuteGraphOptions::new(&registry, &host);
    options.capabilities = ExecutionCapabilities {
        read: true,
        write: true,
    };
    let err = execute_graph(&passthrough_graph(), options).unwrap_err();
    assert!(matches!(err, ExecutionError::WriteCapabilityRejected));
}
