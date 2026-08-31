//! Graph library parse tests.

use imp_graph_library::{parse_graph_library_json, GraphLibraryParseError};
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn parses_valid_library_fixture() {
    let text = fs::read_to_string(fixture_path("passthrough.json")).unwrap();
    let library = parse_graph_library_json(&text).unwrap();
    assert_eq!(library.id, "example.query-patterns");
    assert_eq!(library.definitions.len(), 1);
    assert_eq!(library.definitions[0].id, "passthrough");
    assert_eq!(library.definitions[0].name.as_deref(), Some("Passthrough rows"));
}

#[test]
fn accepts_empty_definitions_array() {
    let library =
        parse_graph_library_json(r#"{"id":"empty.lib","definitions":[]}"#).unwrap();
    assert!(library.definitions.is_empty());
}

#[test]
fn rejects_duplicate_definition_ids() {
    let err = parse_graph_library_json(
        r#"{
          "id": "dup.lib",
          "definitions": [
            {
              "id": "same",
              "inputs": {},
              "outputs": {},
              "bindings": { "inputs": {}, "outputs": {} },
              "body": { "nodes": {}, "edges": {} }
            },
            {
              "id": "same",
              "inputs": {},
              "outputs": {},
              "bindings": { "inputs": {}, "outputs": {} },
              "body": { "nodes": {}, "edges": {} }
            }
          ]
        }"#,
    )
    .unwrap_err();
    assert!(matches!(err, GraphLibraryParseError(_)));
}

#[test]
fn rejects_primitive_definition_with_implementation() {
    let err = parse_graph_library_json(
        r#"{
          "id": "bad.lib",
          "definitions": [
            {
              "id": "filter",
              "implementation": { "kind": "universal", "id": "filter" },
              "inputs": {},
              "outputs": {},
              "bindings": { "inputs": {}, "outputs": {} },
              "body": { "nodes": {}, "edges": {} }
            }
          ]
        }"#,
    )
    .unwrap_err();
    assert!(err.0.contains("must not declare implementation"));
}
