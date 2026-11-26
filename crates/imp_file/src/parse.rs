//! Imp file JSON parsing and structural validation.

use std::collections::BTreeSet;

use imp_core_types::{ImpFile, NodeDefinition, IMP_FILE_FORMAT};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0}")]
pub struct ImpFileParseError(pub String);

fn fail<T>(message: impl Into<String>) -> Result<T, ImpFileParseError> {
    Err(ImpFileParseError(message.into()))
}

fn require_object<'a>(
    value: &'a Value,
    label: &str,
) -> Result<&'a Map<String, Value>, ImpFileParseError> {
    match value.as_object() {
        Some(object) => Ok(object),
        None => fail(format!("{label} must be an object")),
    }
}

fn require_string<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str, ImpFileParseError> {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(text),
        _ => fail(format!("{label} must be a non-empty string")),
    }
}

fn require_field_object<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    label: &str,
) -> Result<&'a Map<String, Value>, ImpFileParseError> {
    match object.get(field) {
        Some(value) => require_object(value, &format!("{label}.{field}")),
        None => fail(format!("{label}.{field} must be an object")),
    }
}

fn require_port_reference(
    object: &Map<String, Value>,
    field: &str,
    label: &str,
) -> Result<(), ImpFileParseError> {
    let reference = require_field_object(object, field, label)?;
    require_string(reference.get("node"), &format!("{label}.{field}.node"))?;
    require_string(reference.get("port"), &format!("{label}.{field}.port"))?;
    Ok(())
}

fn validate_graph(value: &Value, label: &str) -> Result<(), ImpFileParseError> {
    let graph = require_object(value, label)?;
    let nodes = require_field_object(graph, "nodes", label)?;
    let edges = require_field_object(graph, "edges", label)?;
    for (key, node) in nodes {
        let node = require_object(node, &format!("{label}.nodes[\"{key}\"]"))?;
        if node.get("id").and_then(Value::as_str) != Some(key.as_str()) {
            return fail(format!("{label}.nodes[\"{key}\"].id must equal key"));
        }
    }
    for (key, edge) in edges {
        let edge_label = format!("{label}.edges[\"{key}\"]");
        let edge = require_object(edge, &edge_label)?;
        require_port_reference(edge, "from", &edge_label)?;
        require_port_reference(edge, "to", &edge_label)?;
    }
    Ok(())
}

fn validate_graph_backed(
    raw: &Map<String, Value>,
    definition: &NodeDefinition,
    label: &str,
) -> Result<(), ImpFileParseError> {
    if definition.implementation.is_some() {
        return fail(format!(
            "{label} is graph-backed and must not declare implementation"
        ));
    }
    let Some(bindings) = &definition.bindings else {
        return fail(format!("{label} has a body and must include bindings"));
    };
    validate_graph(&raw["body"], &format!("{label}.body"))?;
    let body = definition.body.as_ref().expect("body present");

    for (direction, ports, bound) in [
        ("input", &definition.inputs, &bindings.inputs),
        ("output", &definition.outputs, &bindings.outputs),
    ] {
        for port_id in ports.keys() {
            if !bound.contains_key(port_id) {
                return fail(format!(
                    "{label} missing {direction} binding for port \"{port_id}\""
                ));
            }
        }
        for port_id in bound.keys() {
            if !ports.contains_key(port_id) {
                return fail(format!(
                    "{label} binds undeclared {direction} port \"{port_id}\""
                ));
            }
        }
        for boundary_id in bound.values() {
            let Some(node) = body.nodes.get(boundary_id) else {
                return fail(format!(
                    "{label} bound {direction} node \"{boundary_id}\" not found in body"
                ));
            };
            if node.node_type != direction {
                return fail(format!(
                    "{label} bound {direction} node \"{boundary_id}\" must have type \"{direction}\""
                ));
            }
        }
    }
    Ok(())
}

fn deserialize<T: DeserializeOwned>(value: &Value, label: &str) -> Result<T, ImpFileParseError> {
    serde_json::from_value(value.clone())
        .map_err(|err| ImpFileParseError(format!("{label}: {err}")))
}

fn validate_element(section: &str, value: &Value, label: &str) -> Result<(), ImpFileParseError> {
    let object = require_object(value, label)?;
    require_string(object.get("id"), &format!("{label}.id"))?;
    match section {
        "types" => {
            require_field_object(object, "type", label)?;
            if object
                .get("params")
                .is_some_and(|params| !params.is_array())
            {
                return fail(format!("{label}.params must be an array"));
            }
        }
        "definitions" => {
            require_field_object(object, "inputs", label)?;
            require_field_object(object, "outputs", label)?;
            let definition: NodeDefinition = deserialize(value, label)?;
            if definition.body.is_some() {
                validate_graph_backed(object, &definition, label)?;
            } else if definition.bindings.is_some() {
                return fail(format!("{label} declares bindings without a body"));
            }
        }
        "nodes" => {
            require_string(object.get("type"), &format!("{label}.type"))?;
            require_field_object(object, "inputs", label)?;
        }
        "edges" => {
            require_port_reference(object, "from", label)?;
            require_port_reference(object, "to", label)?;
        }
        _ => unreachable!("known section"),
    }
    Ok(())
}

/// Validates an already-parsed JSON value as an Imp file.
pub fn parse_imp_file(value: &Value) -> Result<ImpFile, ImpFileParseError> {
    let object = require_object(value, "Imp file")?;
    if let Some(format) = object.get("format") {
        if format.as_str() != Some(IMP_FILE_FORMAT) {
            return fail("format must be \"imp\" when present");
        }
    }
    if object.contains_key("id") {
        require_string(object.get("id"), "id")?;
    }
    for section in ["types", "definitions", "nodes", "edges"] {
        let Some(elements) = object.get(section) else {
            continue;
        };
        let Some(elements) = elements.as_array() else {
            return fail(format!("{section} must be an array"));
        };
        let mut seen = BTreeSet::new();
        for (index, element) in elements.iter().enumerate() {
            validate_element(section, element, &format!("{section}[{index}]"))?;
            let id = element["id"].as_str().unwrap_or_default();
            if !seen.insert(id.to_string()) {
                return fail(format!("duplicate {section} id \"{id}\""));
            }
        }
    }
    deserialize(value, "Imp file")
}

/// Parses UTF-8 JSON text and validates it as an Imp file.
pub fn parse_imp_file_json(text: &str) -> Result<ImpFile, ImpFileParseError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|err| ImpFileParseError(format!("invalid JSON: {err}")))?;
    parse_imp_file(&value)
}
