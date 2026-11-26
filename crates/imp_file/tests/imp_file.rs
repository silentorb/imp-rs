//! Imp file parsing, validation and conversions. Mirrors imp-ts `imp-file.test.ts`.

use imp_core_types::{resolved_nodes, Edge, Graph, ImpFileSection, Node, PortReference};
use imp_file::{
    assert_imp_file_sections, imp_file_from_graph, imp_file_graph, imp_file_graph_source,
    parse_imp_file, parse_imp_file_json, serialize_imp_file,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn echo() -> Value {
    json!({
        "id": "echo",
        "typeParams": [{ "id": "A", "bounds": [{ "id": "Audio" }] }],
        "inputs": { "audio": { "id": "audio", "type": { "param": "A" } } },
        "outputs": { "audio": { "id": "audio", "type": { "param": "A" } } },
        "bindings": { "inputs": { "audio": "in" }, "outputs": { "audio": "out" } },
        "body": {
            "nodes": {
                "in": { "id": "in", "type": "input", "inputs": {} },
                "out": { "id": "out", "type": "output", "inputs": {} }
            },
            "edges": { "e1": { "from": { "node": "in", "port": "value" }, "to": { "node": "out", "port": "value" } } }
        }
    })
}

fn example() -> Value {
    json!({
        "format": "imp",
        "id": "example.audio",
        "types": [{ "id": "Audio", "type": { "union": [{ "id": "audio_mono" }, { "id": "audio_stereo" }] } }],
        "definitions": [
            echo(),
            { "id": "sine", "inputs": {}, "outputs": { "audio": { "id": "audio", "type": { "id": "audio_mono" } } } }
        ],
        "nodes": [
            { "id": "src", "type": "sine", "inputs": {} },
            { "id": "fx", "type": "echo", "inputs": {} }
        ],
        "edges": [{ "id": "e1", "from": { "node": "src", "port": "audio" }, "to": { "node": "fx", "port": "audio" } }]
    })
}

fn rejects(value: Value, message: &str) {
    let err = parse_imp_file(&value).unwrap_err();
    assert!(
        err.0.contains(message),
        "expected {message:?} in {:?}",
        err.0
    );
}

fn with_echo(patch: impl FnOnce(&mut Value)) -> Value {
    let mut definition = echo();
    patch(&mut definition);
    json!({ "definitions": [definition] })
}

#[test]
fn parses_every_section_including_primitive_and_graph_backed_definitions() {
    let file = parse_imp_file_json(&example().to_string()).unwrap();
    assert_eq!(file.format.as_deref(), Some("imp"));
    assert_eq!(file.types[0].id, "Audio");
    let ids: Vec<_> = file.definitions.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["echo", "sine"]);
    let nodes: Vec<_> = file.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(nodes, ["src", "fx"]);
    assert_eq!(file.edges[0].id, "e1");
}

#[test]
fn an_empty_object_is_an_empty_file() {
    let file = parse_imp_file(&json!({})).unwrap();
    assert!(file.nodes.is_empty() && file.types.is_empty() && file.format.is_none());
}

#[test]
fn rejects_malformed_top_level_fields_and_sections() {
    rejects(json!([]), "Imp file must be an object");
    rejects(json!({ "format": "imp-graph" }), "format must be \"imp\"");
    rejects(json!({ "id": "" }), "id must be a non-empty string");
    rejects(json!({ "nodes": {} }), "nodes must be an array");
}

#[test]
fn rejects_malformed_and_duplicate_elements() {
    rejects(
        json!({ "nodes": [{ "id": "a", "inputs": {} }] }),
        "nodes[0].type",
    );
    rejects(
        json!({ "edges": [{ "id": "e", "from": { "node": "a" }, "to": { "node": "b", "port": "x" } }] }),
        "edges[0].from.port",
    );
    rejects(
        json!({ "types": [{ "id": "T" }] }),
        "types[0].type must be an object",
    );
    rejects(
        json!({ "nodes": [
            { "id": "a", "type": "t", "inputs": {} },
            { "id": "a", "type": "t", "inputs": {} }
        ] }),
        "duplicate nodes id \"a\"",
    );
}

#[test]
fn validates_graph_backed_definitions() {
    rejects(
        with_echo(|d| d["implementation"] = json!({ "kind": "universal", "id": "x" })),
        "must not declare implementation",
    );
    rejects(
        with_echo(|d| {
            d.as_object_mut().unwrap().remove("bindings");
        }),
        "must include bindings",
    );
    rejects(
        with_echo(|d| d["bindings"] = json!({ "inputs": {}, "outputs": { "audio": "out" } })),
        "missing input binding for port \"audio\"",
    );
    rejects(
        with_echo(|d| {
            d["bindings"] = json!({ "inputs": { "audio": "out" }, "outputs": { "audio": "out" } })
        }),
        "must have type \"input\"",
    );
    rejects(
        json!({ "definitions": [{ "id": "p", "inputs": {}, "outputs": {}, "bindings": { "inputs": {}, "outputs": {} } }] }),
        "bindings without a body",
    );
}

#[test]
fn serializes_in_canonical_order_and_omits_empty_sections() {
    let mut value = example();
    value["types"] = json!([]);
    let file = parse_imp_file(&value).unwrap();
    let text = serialize_imp_file(&file).unwrap();
    // Top-level keys appear in canonical order; `types` (empty) is omitted.
    let positions: Vec<usize> = ["\"format\"", "\"id\"", "\"definitions\"", "\"nodes\"", "\"edges\""]
        .iter()
        .map(|key| text.find(key).expect(key))
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]), "{text}");
    assert!(!text.contains("\"types\""));
    assert_eq!(parse_imp_file_json(&text).unwrap(), file);
}

#[test]
fn nodes_and_edges_become_a_graph_source() {
    let file = parse_imp_file(&example()).unwrap();
    let source = imp_file_graph_source(&file);
    let ids: Vec<_> = resolved_nodes(source).iter().map(|n| n.id.clone()).collect();
    assert_eq!(ids, ["src", "fx"]);
    assert_eq!(
        source.edges.get_edge(&"e1".to_string()).unwrap().to.node,
        "fx"
    );
    assert_eq!(imp_file_graph(&file).nodes.len(), 2);
}

#[test]
fn a_graph_round_trips_through_an_imp_file() {
    let graph = Graph {
        nodes: BTreeMap::from([(
            "a".to_string(),
            Node {
                id: "a".into(),
                title: None,
                node_type: "t".into(),
                type_args: vec![],
                inputs: BTreeMap::new(),
            },
        )]),
        edges: BTreeMap::from([(
            "e".to_string(),
            Edge {
                from: PortReference {
                    node: "a".into(),
                    port: "x".into(),
                },
                to: PortReference {
                    node: "b".into(),
                    port: "y".into(),
                },
                order: None,
            },
        )]),
    };
    let file = imp_file_from_graph(&graph);
    assert_eq!(file.edges[0].id, "e");
    assert_eq!(imp_file_graph(&file), graph);
}

#[test]
fn section_restrictions_reject_disallowed_sections() {
    let topology = imp_file_from_graph(&Graph {
        nodes: BTreeMap::new(),
        edges: BTreeMap::new(),
    });
    let allowed = [ImpFileSection::Nodes, ImpFileSection::Edges];
    assert!(assert_imp_file_sections(&topology, &allowed).is_ok());
    let err = assert_imp_file_sections(&parse_imp_file(&example()).unwrap(), &allowed).unwrap_err();
    assert!(err.to_string().contains("section \"types\" is not allowed"));
}
