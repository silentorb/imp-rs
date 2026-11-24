//! JSON field names must match imp-spec / imp-ts (camelCase) exactly.

use imp_core_types::{Graph, NodeDefinition, NodeLibrary, SignalType};
use serde_json::{json, Value};

fn round_trip<T>(value: &Value) -> (T, Value)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let parsed: T = serde_json::from_value(value.clone()).expect("parse TS-shaped JSON");
    let reserialized = serde_json::to_value(&parsed).expect("serialize");
    (parsed, reserialized)
}

#[test]
fn graph_with_type_args_round_trips_unchanged() {
    let ts_graph = json!({
        "nodes": {
            "in": {
                "id": "in",
                "title": "List input",
                "type": "input",
                "typeArgs": [{ "id": "list", "args": [{ "id": "number" }] }],
                "inputs": { "label": "items" }
            },
            "out": {
                "id": "out",
                "type": "output",
                "inputs": {}
            }
        },
        "edges": {
            "e1": {
                "from": { "node": "in", "port": "value" },
                "to": { "node": "out", "port": "value" },
                "order": 0
            }
        }
    });

    let (graph, reserialized): (Graph, _) = round_trip(&ts_graph);
    assert_eq!(
        graph.nodes["in"].type_args,
        vec![SignalType::Concrete {
            id: "list".into(),
            args: vec![SignalType::Concrete {
                id: "number".into(),
                args: vec![],
            }],
        }]
    );
    assert_eq!(reserialized, ts_graph);
}

#[test]
fn node_definition_with_default_value_and_type_params_round_trips_unchanged() {
    let ts_definition = json!({
        "id": "scale",
        "name": "Scale",
        "typeParams": [{ "id": "T", "bounds": ["Scalar"] }],
        "implementation": {
            "kind": "dispatch",
            "paramIndex": 0,
            "cases": [{ "match": { "id": "number" }, "implementation": "scale_number" }],
            "default": "scale_any"
        },
        "inputs": {
            "value": { "id": "value", "type": { "param": "T" } },
            "factor": { "id": "factor", "type": { "id": "number" }, "defaultValue": 1.0 },
            "items": { "id": "items", "type": { "param": "T" }, "variadic": true }
        },
        "outputs": {
            "result": { "id": "result", "type": { "param": "T" } }
        }
    });

    let (definition, reserialized): (NodeDefinition, _) = round_trip(&ts_definition);
    assert_eq!(definition.type_params.len(), 1);
    assert!(definition.inputs["factor"].default_value.is_some());
    assert_eq!(reserialized, ts_definition);
}

#[test]
fn graph_backed_library_round_trips_unchanged() {
    let ts_library = json!({
        "id": "lib",
        "definitions": [{
            "id": "passthrough",
            "typeParams": [{ "id": "T" }],
            "inputs": { "x": { "id": "x", "type": { "param": "T" } } },
            "outputs": { "y": { "id": "y", "type": { "param": "T" } } },
            "body": {
                "nodes": {
                    "in": {
                        "id": "in",
                        "type": "input",
                        "typeArgs": [{ "param": "T" }],
                        "inputs": {}
                    },
                    "out": {
                        "id": "out",
                        "type": "output",
                        "typeArgs": [{ "param": "T" }],
                        "inputs": {}
                    }
                },
                "edges": {
                    "e": {
                        "from": { "node": "in", "port": "value" },
                        "to": { "node": "out", "port": "value" }
                    }
                }
            },
            "bindings": {
                "inputs": { "x": "in" },
                "outputs": { "y": "out" }
            }
        }]
    });

    let (_, reserialized): (NodeLibrary, _) = round_trip(&ts_library);
    assert_eq!(reserialized, ts_library);
}

#[test]
fn snake_case_keys_are_not_emitted() {
    let ts_definition = json!({
        "id": "d",
        "typeParams": [{ "id": "T" }],
        "inputs": { "a": { "id": "a", "type": { "id": "number" }, "defaultValue": 2.0 } },
        "outputs": {}
    });
    let (_, out): (NodeDefinition, _) = round_trip(&ts_definition);
    let text = out.to_string();
    for key in ["type_params", "default_value", "type_args", "param_index"] {
        assert!(!text.contains(key), "unexpected snake_case key {key} in {text}");
    }
}
