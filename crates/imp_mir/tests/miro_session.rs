use imp_core_types::{Graph, Node, PrimitiveValue};
use imp_mir::{
    default_miro_graph, unknown_kind_fn, wire_sync_graph, MiroOpenOptions, MiroSession,
    StoreRegistry, WireError, MIR_SYNC_STORE, PORT_STORE_ID,
};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

fn kind_from_aura_json(_path: &str, content: &str) -> String {
    // Minimal `"type":"..."` extract for tests (no serde_json).
    let Some(rest) = content.split("\"type\"").nth(1) else {
        return "unknown".to_string();
    };
    let Some(rest) = rest.split('"').nth(1) else {
        return "unknown".to_string();
    };
    if rest.is_empty() {
        "unknown".to_string()
    } else {
        rest.to_string()
    }
}

#[test]
fn cold_sync_and_pull_round_trip() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    fs::write(
        root.join("demo.json"),
        r#"{"type":"imp-graph","nodes":{},"edges":{}}"#,
    )
    .expect("write");
    fs::write(root.join("notes.txt"), "hello").expect("write");

    let kind_fn = Arc::new(kind_from_aura_json);
    let session = MiroSession::open_in_memory(root, kind_fn, false).expect("open session");

    let demo = session
        .pull_record("demo.json")
        .expect("pull")
        .expect("present");
    assert_eq!(demo.kind, "imp-graph");
    assert!(demo.content.contains("imp-graph"));

    let notes = session
        .pull_record("notes.txt")
        .expect("pull")
        .expect("present");
    assert_eq!(notes.kind, "unknown");
    assert_eq!(notes.content, "hello");

    assert!(session
        .pull_record("missing.json")
        .expect("pull")
        .is_none());
}

#[test]
fn wire_rejects_unknown_node_type() {
    let registry = StoreRegistry::new();
    let mut nodes = BTreeMap::new();
    nodes.insert(
        "x".to_string(),
        Node {
            id: "x".to_string(),
            node_type: "not.mir".to_string(),
            type_args: Vec::new(),
            inputs: BTreeMap::from([(
                PORT_STORE_ID.to_string(),
                PrimitiveValue::String("authoring".to_string()),
            )]),
        },
    );
    let graph = Graph {
        nodes,
        edges: BTreeMap::new(),
    };
    let err = wire_sync_graph(&graph, &registry).expect_err("should fail");
    assert!(matches!(err, WireError::UnknownNodeType { .. }));
}

#[test]
fn default_miro_graph_wires() {
    let graph = default_miro_graph("authoring", "projection");
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.nodes["authoring"].node_type, MIR_SYNC_STORE);
    let _ = unknown_kind_fn();
}

#[test]
fn open_options_file_db() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path().join("content");
    fs::create_dir_all(&root).expect("mkdir");
    fs::write(root.join("a.json"), r#"{"type":"notation"}"#).expect("write");
    let db = dir.path().join("proj.db");

    let session = MiroSession::open(
        MiroOpenOptions::new(&root, &db)
            .with_kind_fn(Arc::new(kind_from_aura_json))
            .with_start_watch(false),
    )
    .expect("open");

    let rec = session.pull_record("a.json").expect("pull").expect("row");
    assert_eq!(rec.kind, "notation");
}
