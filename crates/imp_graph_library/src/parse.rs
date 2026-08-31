//! Graph library JSON parsing and validation.

use imp_core_types::{is_graph_backed_definition, GraphLibrary, NodeDefinition};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0}")]
pub struct GraphLibraryParseError(pub String);

fn require_string(value: &Value, label: &str) -> Result<String, GraphLibraryParseError> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Ok(text.clone()),
        _ => Err(GraphLibraryParseError(format!(
            "{label} must be a non-empty string"
        ))),
    }
}

fn validate_graph(graph: &Value, label: &str) -> Result<(), GraphLibraryParseError> {
    let Some(obj) = graph.as_object() else {
        return Err(GraphLibraryParseError(format!("{label} must be an object")));
    };
    let Some(nodes) = obj.get("nodes") else {
        return Err(GraphLibraryParseError(format!(
            "{label}.nodes must be an object"
        )));
    };
    let Some(edges) = obj.get("edges") else {
        return Err(GraphLibraryParseError(format!(
            "{label}.edges must be an object"
        )));
    };
    if !nodes.is_object() {
        return Err(GraphLibraryParseError(format!(
            "{label}.nodes must be an object"
        )));
    }
    if !edges.is_object() {
        return Err(GraphLibraryParseError(format!(
            "{label}.edges must be an object"
        )));
    }
    if let Some(node_map) = nodes.as_object() {
        for (key, node) in node_map {
            let Some(node_obj) = node.as_object() else {
                return Err(GraphLibraryParseError(format!(
                    "{label}.nodes[\"{key}\"] must be an object"
                )));
            };
            if node_obj.get("id").and_then(Value::as_str) != Some(key.as_str()) {
                return Err(GraphLibraryParseError(format!(
                    "{label}.nodes[\"{key}\"].id must equal key"
                )));
            }
        }
    }
    Ok(())
}

fn validate_definition(raw: &Value, index: usize) -> Result<NodeDefinition, GraphLibraryParseError> {
    let Some(obj) = raw.as_object() else {
        return Err(GraphLibraryParseError(format!(
            "definitions[{index}] must be an object"
        )));
    };

    let id = require_string(
        obj.get("id").unwrap_or(&Value::Null),
        &format!("definitions[{index}].id"),
    )?;

    if obj.contains_key("implementation") {
        return Err(GraphLibraryParseError(format!(
            "definitions[{index}] \"{id}\" must not declare implementation"
        )));
    }

    let body = obj.get("body").ok_or_else(|| {
        GraphLibraryParseError(format!(
            "definitions[{index}] \"{id}\" must include body"
        ))
    })?;
    validate_graph(body, &format!("definitions[{index}] \"{id}\".body"))?;

    let bindings = obj.get("bindings").ok_or_else(|| {
        GraphLibraryParseError(format!(
            "definitions[{index}] \"{id}\" must include bindings"
        ))
    })?;
    let Some(bindings_obj) = bindings.as_object() else {
        return Err(GraphLibraryParseError(format!(
            "definitions[{index}] \"{id}\" must include bindings"
        )));
    };
    let inputs = bindings_obj
        .get("inputs")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" must include bindings.inputs"
            ))
        })?;
    let outputs = bindings_obj
        .get("outputs")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" must include bindings.outputs"
            ))
        })?;

    let def_inputs = obj
        .get("inputs")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" must include inputs and outputs"
            ))
        })?;
    let def_outputs = obj
        .get("outputs")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" must include inputs and outputs"
            ))
        })?;

    for port_id in def_inputs.keys() {
        if !inputs.contains_key(port_id) {
            return Err(GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" missing input binding for port \"{port_id}\""
            )));
        }
    }
    for port_id in def_outputs.keys() {
        if !outputs.contains_key(port_id) {
            return Err(GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" missing output binding for port \"{port_id}\""
            )));
        }
    }

    let definition: NodeDefinition = serde_json::from_value(raw.clone()).map_err(|err| {
        GraphLibraryParseError(format!(
            "definitions[{index}] \"{id}\" invalid shape: {err}"
        ))
    })?;

    for boundary_id in inputs.values().filter_map(Value::as_str) {
        let Some(node) = definition.body.as_ref().and_then(|g| g.nodes.get(boundary_id)) else {
            return Err(GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" bound input node \"{boundary_id}\" not found in body"
            )));
        };
        if node.node_type != "input" {
            return Err(GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" bound input node \"{boundary_id}\" must have type \"input\""
            )));
        }
    }

    for boundary_id in outputs.values().filter_map(Value::as_str) {
        let Some(node) = definition.body.as_ref().and_then(|g| g.nodes.get(boundary_id)) else {
            return Err(GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" bound output node \"{boundary_id}\" not found in body"
            )));
        };
        if node.node_type != "output" {
            return Err(GraphLibraryParseError(format!(
                "definitions[{index}] \"{id}\" bound output node \"{boundary_id}\" must have type \"output\""
            )));
        }
    }

    if !is_graph_backed_definition(&definition) {
        return Err(GraphLibraryParseError(format!(
            "definitions[{index}] \"{id}\" is not graph-backed"
        )));
    }

    Ok(definition)
}

pub fn parse_graph_library_json(text: &str) -> Result<GraphLibrary, GraphLibraryParseError> {
    let parsed: Value = serde_json::from_str(text).map_err(|err| {
        GraphLibraryParseError(format!("invalid JSON: {err}"))
    })?;

    parse_graph_library(parsed)
}

pub fn parse_graph_library(value: Value) -> Result<GraphLibrary, GraphLibraryParseError> {
    let Some(obj) = value.as_object() else {
        return Err(GraphLibraryParseError(
            "graph library must be a JSON object".into(),
        ));
    };

    let id = require_string(obj.get("id").unwrap_or(&Value::Null), "id")?;
    let definitions_value = obj.get("definitions").ok_or_else(|| {
        GraphLibraryParseError("definitions must be an array".into())
    })?;
    let Some(definitions_array) = definitions_value.as_array() else {
        return Err(GraphLibraryParseError("definitions must be an array".into()));
    };

    let mut definitions = Vec::with_capacity(definitions_array.len());
    let mut seen = std::collections::BTreeSet::new();
    for (index, raw) in definitions_array.iter().enumerate() {
        let definition = validate_definition(raw, index)?;
        if !seen.insert(definition.id.clone()) {
            return Err(GraphLibraryParseError(format!(
                "duplicate definition id \"{}\"",
                definition.id
            )));
        }
        definitions.push(definition);
    }

    Ok(GraphLibrary { id, definitions })
}
