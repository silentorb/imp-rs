//! In-memory collection evaluation.
//! Spec: imp-ts/docs/features/execution.md

use crate::error::ExecutionError;
use crate::host::{ExecutionHost, ExecutionRow};
use crate::resolve::{
    require_node, resolve_input, EdgeTargetKey, ResolvedInput,
};
use imp_core_types::{BoundaryBindings, Graph, NodeId, PortId, PrimitiveValue};
use imp_graph_resolve::{is_graph_backed_node_type, subgraph_key, ExecutionProgram};
use imp_registry::{get_node_definition, Registry};
use crate::resolve::index_edges_by_target;
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub struct CompositeFrame<'a> {
    pub instance_id: NodeId,
    pub bindings: BoundaryBindings,
    pub parent_graph: &'a Graph,
    pub parent_edges: &'a BTreeMap<EdgeTargetKey, imp_core_types::Edge>,
}

pub struct EvalContext<'a, H: ExecutionHost> {
    pub graph: &'a Graph,
    pub registry: &'a Registry,
    pub host: &'a H,
    pub edges_by_target: &'a BTreeMap<EdgeTargetKey, imp_core_types::Edge>,
    pub source_node_id: NodeId,
    pub execution_program: Option<&'a ExecutionProgram>,
    pub composite: Option<CompositeFrame<'a>>,
}

fn require_number(value: &JsonValue, label: &str) -> Result<f64, ExecutionError> {
    match value {
        JsonValue::Number(n) => n
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| ExecutionError::message(format!("{label} must be a finite number"))),
        _ => Err(ExecutionError::message(format!("{label} must be a finite number"))),
    }
}

fn require_string(value: &JsonValue, label: &str) -> Result<String, ExecutionError> {
    match value {
        JsonValue::String(s) => Ok(s.clone()),
        _ => Err(ExecutionError::message(format!("{label} must be a string"))),
    }
}

fn primitive_to_json(value: &PrimitiveValue) -> JsonValue {
    match value {
        PrimitiveValue::String(s) => JsonValue::String(s.clone()),
        PrimitiveValue::Number(n) => {
            JsonValue::Number(serde_json::Number::from_f64(*n).unwrap_or_else(|| 0.into()))
        }
        PrimitiveValue::Bool(b) => JsonValue::Bool(*b),
        PrimitiveValue::Null => JsonValue::Null,
    }
}

fn resolve_literal_or_throw<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    node_id: &NodeId,
    port_id: &PortId,
    label: &str,
) -> Result<JsonValue, ExecutionError> {
    let resolved = resolve_input(
        ctx.graph,
        ctx.registry,
        ctx.edges_by_target,
        node_id,
        port_id,
    )?;
    match resolved {
        ResolvedInput::Wire { from } => {
            let from_type = &require_node(ctx.graph, &from.node)?.node_type;
            if from_type == "literal" || from_type == "parameter" {
                let lit = resolve_input(
                    ctx.graph,
                    ctx.registry,
                    ctx.edges_by_target,
                    &from.node,
                    &"value".into(),
                )?;
                if let ResolvedInput::Literal { value } = lit {
                    return Ok(primitive_to_json(&value));
                }
            }
            Err(ExecutionError::message(format!(
                "{label} on \"{node_id}\" must resolve to a literal"
            )))
        }
        ResolvedInput::Literal { value } => Ok(primitive_to_json(&value)),
    }
}

fn row_column(row: &ExecutionRow, column: &str) -> JsonValue {
    if column == "id" {
        return JsonValue::String(row.id.clone());
    }
    if column == "is_archived" {
        return JsonValue::Bool(row.is_archived.unwrap_or(false));
    }
    if let Some(stripped) = column.strip_prefix("properties.") {
        return row
            .properties
            .get(stripped)
            .cloned()
            .unwrap_or(JsonValue::Null);
    }
    row.properties
        .get(column)
        .cloned()
        .unwrap_or(JsonValue::Null)
}

fn values_equal(a: &JsonValue, b: &JsonValue) -> bool {
    if a == b {
        return true;
    }
    match (a, b) {
        (JsonValue::Number(a), JsonValue::Number(b)) => {
            a.as_f64().unwrap_or(f64::NAN) == b.as_f64().unwrap_or(f64::NAN)
        }
        _ => false,
    }
}

fn json_to_f64(value: &JsonValue) -> f64 {
    match value {
        JsonValue::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        JsonValue::String(s) => s.parse().unwrap_or(f64::NAN),
        JsonValue::Bool(true) => 1.0,
        JsonValue::Bool(false) => 0.0,
        JsonValue::Null => f64::NAN,
        _ => f64::NAN,
    }
}

fn is_nullish(value: &JsonValue) -> bool {
    matches!(value, JsonValue::Null)
}

fn sort_cmp(a: &JsonValue, b: &JsonValue, sign: i32) -> std::cmp::Ordering {
    if a == b {
        return std::cmp::Ordering::Equal;
    }
    if is_nullish(a) {
        return std::cmp::Ordering::Greater;
    }
    if is_nullish(b) {
        return std::cmp::Ordering::Less;
    }
    let less = match (a, b) {
        (JsonValue::Number(a), JsonValue::Number(b)) => {
            a.as_f64().unwrap_or(0.0) < b.as_f64().unwrap_or(0.0)
        }
        (JsonValue::String(a), JsonValue::String(b)) => a < b,
        _ => json_to_f64(a) < json_to_f64(b),
    };
    if less {
        if sign == -1 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    } else if sign == -1 {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}

fn eval_expression<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &NodeId,
    port_id: &PortId,
    current_row: Option<&ExecutionRow>,
) -> Result<JsonValue, ExecutionError> {
    let resolved = resolve_input(
        ctx.graph,
        ctx.registry,
        ctx.edges_by_target,
        node_id,
        port_id,
    )?;
    match resolved {
        ResolvedInput::Literal { value } => Ok(primitive_to_json(&value)),
        ResolvedInput::Wire { from } => {
            let from_node = require_node(ctx.graph, &from.node)?;
            match from_node.node_type.as_str() {
                "column" => {
                    let row = current_row.ok_or_else(|| {
                        ExecutionError::message(format!(
                            "column \"{}\" requires a row context",
                            from.node
                        ))
                    })?;
                    let name_raw = from_node.inputs.get("name").ok_or_else(|| {
                        ExecutionError::message("column.name must be a string".to_string())
                    })?;
                    let column_name = match name_raw {
                        PrimitiveValue::String(s) => s.as_str(),
                        _ => {
                            return Err(ExecutionError::message(
                                "column.name must be a string".to_string(),
                            ));
                        }
                    };
                    Ok(row_column(row, column_name))
                }
                "literal" | "parameter" => {
                    let value = from_node
                        .inputs
                        .get("value")
                        .map(primitive_to_json)
                        .unwrap_or(JsonValue::Null);
                    Ok(value)
                }
                _ if port_id == "result" || port_id == "value" => eval_predicate_value(
                    ctx,
                    visiting,
                    &from.node,
                    current_row,
                ),
                other => Err(ExecutionError::message(format!(
                    "Unsupported expression node type \"{other}\" on \"{}\"",
                    from.node
                ))),
            }
        }
    }
}

fn eval_predicate_value<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &NodeId,
    current_row: Option<&ExecutionRow>,
) -> Result<JsonValue, ExecutionError> {
    let node = require_node(ctx.graph, node_id)?;
    match node.node_type.as_str() {
        "equals" => {
            let left = eval_expression(ctx, visiting, node_id, &"left".into(), current_row)?;
            let right = eval_expression(ctx, visiting, node_id, &"right".into(), current_row)?;
            Ok(JsonValue::Bool(values_equal(&left, &right)))
        }
        "not_equals" => {
            let left = eval_expression(ctx, visiting, node_id, &"left".into(), current_row)?;
            let right = eval_expression(ctx, visiting, node_id, &"right".into(), current_row)?;
            Ok(JsonValue::Bool(!values_equal(&left, &right)))
        }
        "less_than" => {
            let left = json_to_f64(&eval_expression(
                ctx,
                visiting,
                node_id,
                &"left".into(),
                current_row,
            )?);
            let right = json_to_f64(&eval_expression(
                ctx,
                visiting,
                node_id,
                &"right".into(),
                current_row,
            )?);
            Ok(JsonValue::Bool(left < right))
        }
        "greater_than" => {
            let left = json_to_f64(&eval_expression(
                ctx,
                visiting,
                node_id,
                &"left".into(),
                current_row,
            )?);
            let right = json_to_f64(&eval_expression(
                ctx,
                visiting,
                node_id,
                &"right".into(),
                current_row,
            )?);
            Ok(JsonValue::Bool(left > right))
        }
        "contains" => {
            let haystack = eval_expression(ctx, visiting, node_id, &"haystack".into(), current_row)?;
            let needle = eval_expression(ctx, visiting, node_id, &"needle".into(), current_row)?;
            let result = match (haystack, needle) {
                (JsonValue::String(h), JsonValue::String(n)) => {
                    h.to_lowercase().contains(&n.to_lowercase())
                }
                _ => false,
            };
            Ok(JsonValue::Bool(result))
        }
        "and" => {
            let left = resolve_input(
                ctx.graph,
                ctx.registry,
                ctx.edges_by_target,
                node_id,
                &"left".into(),
            )?;
            let right = resolve_input(
                ctx.graph,
                ctx.registry,
                ctx.edges_by_target,
                node_id,
                &"right".into(),
            )?;
            let (ResolvedInput::Wire { from: left_from }, ResolvedInput::Wire { from: right_from }) =
                (left, right)
            else {
                return Err(ExecutionError::message(format!(
                    "and on \"{node_id}\" requires wired boolean inputs"
                )));
            };
            let left_val = eval_predicate(ctx, visiting, &left_from.node, current_row)?;
            let right_val = eval_predicate(ctx, visiting, &right_from.node, current_row)?;
            Ok(JsonValue::Bool(left_val && right_val))
        }
        "or" => {
            let left = resolve_input(
                ctx.graph,
                ctx.registry,
                ctx.edges_by_target,
                node_id,
                &"left".into(),
            )?;
            let right = resolve_input(
                ctx.graph,
                ctx.registry,
                ctx.edges_by_target,
                node_id,
                &"right".into(),
            )?;
            let (ResolvedInput::Wire { from: left_from }, ResolvedInput::Wire { from: right_from }) =
                (left, right)
            else {
                return Err(ExecutionError::message(format!(
                    "or on \"{node_id}\" requires wired boolean inputs"
                )));
            };
            let left_val = eval_predicate(ctx, visiting, &left_from.node, current_row)?;
            let right_val = eval_predicate(ctx, visiting, &right_from.node, current_row)?;
            Ok(JsonValue::Bool(left_val || right_val))
        }
        "not" => {
            let inner = resolve_input(
                ctx.graph,
                ctx.registry,
                ctx.edges_by_target,
                node_id,
                &"value".into(),
            )?;
            let ResolvedInput::Wire { from } = inner else {
                return Err(ExecutionError::message(format!(
                    "not on \"{node_id}\" requires wired value input"
                )));
            };
            let val = eval_predicate(ctx, visiting, &from.node, current_row)?;
            Ok(JsonValue::Bool(!val))
        }
        other => Err(ExecutionError::message(format!(
            "Unsupported predicate node type \"{other}\""
        ))),
    }
}

fn json_to_bool(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => false,
        JsonValue::Bool(b) => *b,
        JsonValue::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        JsonValue::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn eval_predicate<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &NodeId,
    current_row: Option<&ExecutionRow>,
) -> Result<bool, ExecutionError> {
    let value = eval_predicate_value(ctx, visiting, node_id, current_row)?;
    Ok(json_to_bool(&value))
}

fn input_port_for_bound_node(bindings: &BoundaryBindings, node_id: &NodeId) -> Option<PortId> {
    bindings
        .inputs
        .iter()
        .find_map(|(port_id, bound_id)| (bound_id == node_id).then(|| port_id.clone()))
}

fn eval_composite_collection_port<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    instance_id: &NodeId,
    output_port: &PortId,
) -> Result<Vec<ExecutionRow>, ExecutionError> {
    let program = ctx
        .execution_program
        .ok_or_else(|| ExecutionError::message(format!(
            "Composite node \"{instance_id}\" requires an execution program"
        )))?;

    let instance = require_node(ctx.graph, instance_id)?;
    let definition = get_node_definition(ctx.registry, &instance.node_type).ok_or_else(|| {
        ExecutionError::message(format!("Node \"{instance_id}\" is not a graph-backed definition"))
    })?;
    let bindings = definition.bindings.as_ref().ok_or_else(|| {
        ExecutionError::message(format!("Node \"{instance_id}\" is not a graph-backed definition"))
    })?;

    let key = subgraph_key(&definition.id, Some(&instance.type_args));
    let body = program.subgraphs.get(&key).ok_or_else(|| {
        ExecutionError::message(format!(
            "Missing shared subgraph for definition \"{}\"",
            definition.id
        ))
    })?;

    let boundary_out = bindings.outputs.get(output_port).ok_or_else(|| {
        ExecutionError::message(format!(
            "Composite \"{instance_id}\" has no output binding for port \"{output_port}\""
        ))
    })?;

    let inner_edges = index_edges_by_target(body)?;
    let inner_ctx = EvalContext {
        graph: body,
        registry: ctx.registry,
        host: ctx.host,
        edges_by_target: &inner_edges,
        source_node_id: ctx.source_node_id.clone(),
        execution_program: ctx.execution_program,
        composite: Some(CompositeFrame {
            instance_id: instance_id.clone(),
            bindings: bindings.clone(),
            parent_graph: ctx.graph,
            parent_edges: ctx.edges_by_target,
        }),
    };

    eval_collection_port(&inner_ctx, visiting, boundary_out, &"value".into(), None)
}

fn follow_collection_port_on_parent<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    instance_id: &NodeId,
    port_id: &PortId,
    current_row: Option<&ExecutionRow>,
) -> Result<Vec<ExecutionRow>, ExecutionError> {
    let frame = ctx.composite.as_ref().ok_or_else(|| {
        ExecutionError::message("composite input requires parent frame".to_string())
    })?;
    let resolved = resolve_input(
        frame.parent_graph,
        ctx.registry,
        frame.parent_edges,
        instance_id,
        port_id,
    )?;
    match resolved {
        ResolvedInput::Literal { .. } => Err(ExecutionError::message(format!(
            "composite.{port_id} on \"{instance_id}\" must be wired, not a literal"
        ))),
        ResolvedInput::Wire { from } => {
            let parent_ctx = EvalContext {
                graph: frame.parent_graph,
                registry: ctx.registry,
                host: ctx.host,
                edges_by_target: frame.parent_edges,
                source_node_id: ctx.source_node_id.clone(),
                execution_program: ctx.execution_program,
                composite: None,
            };
            eval_collection_port(
                &parent_ctx,
                visiting,
                &from.node,
                &from.port,
                current_row,
            )
        }
    }
}

fn follow_collection_port<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &NodeId,
    port_id: &PortId,
    label: &str,
) -> Result<Vec<ExecutionRow>, ExecutionError> {
    let resolved = resolve_input(
        ctx.graph,
        ctx.registry,
        ctx.edges_by_target,
        node_id,
        port_id,
    )?;
    match resolved {
        ResolvedInput::Literal { .. } => Err(ExecutionError::message(format!(
            "{label} on \"{node_id}\" must be wired, not a literal"
        ))),
        ResolvedInput::Wire { from } => {
            eval_collection_port(ctx, visiting, &from.node, &from.port, None)
        }
    }
}

fn project_rows(rows: &[ExecutionRow], columns_csv: &str) -> Result<Vec<ExecutionRow>, ExecutionError> {
    let cols: Vec<&str> = columns_csv
        .split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect();
    if cols.is_empty() {
        return Err(ExecutionError::message(
            "project.columns must list at least one column".to_string(),
        ));
    }
    Ok(rows
        .iter()
        .map(|row| {
            let mut projected = ExecutionRow {
                id: row.id.clone(),
                properties: BTreeMap::new(),
                is_archived: row.is_archived,
            };
            for col in &cols {
                if *col == "id" {
                    projected.id = row_column(row, col)
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                } else if *col == "is_archived" {
                    projected.is_archived = Some(row_column(row, col).as_bool().unwrap_or(false));
                } else {
                    let key = col.strip_prefix("properties.").unwrap_or(col);
                    projected
                        .properties
                        .insert(key.to_string(), row_column(row, col));
                }
            }
            projected
        })
        .collect())
}

pub fn eval_collection_port<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &NodeId,
    output_port: &PortId,
    _current_row: Option<&ExecutionRow>,
) -> Result<Vec<ExecutionRow>, ExecutionError> {
    if visiting.contains(node_id) {
        return Err(ExecutionError::CycleDetected(node_id.clone()));
    }
    visiting.insert(node_id.clone());
    let result = eval_collection_port_inner(ctx, visiting, node_id, output_port);
    visiting.remove(node_id);
    result
}

fn eval_collection_port_inner<H: ExecutionHost>(
    ctx: &EvalContext<'_, H>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &NodeId,
    output_port: &PortId,
) -> Result<Vec<ExecutionRow>, ExecutionError> {
    let node = require_node(ctx.graph, node_id)?;
    match node.node_type.as_str() {
        "input" => {
            if let Some(frame) = &ctx.composite {
                if let Some(input_port) = input_port_for_bound_node(&frame.bindings, node_id) {
                    return follow_collection_port_on_parent(
                        ctx,
                        visiting,
                        &frame.instance_id,
                        &input_port,
                        None,
                    );
                }
            }
            if node_id != &ctx.source_node_id {
                return Err(ExecutionError::message(format!(
                    "Unexpected input node \"{node_id}\""
                )));
            }
            if output_port != "value" {
                return Err(ExecutionError::message(format!(
                    "input node \"{node_id}\" only has output port \"value\""
                )));
            }
            ctx.host.list_input_rows()
        }
        "output" => {
            if output_port != "value" {
                return Err(ExecutionError::message(format!(
                    "output node \"{node_id}\" only has input port \"value\""
                )));
            }
            follow_collection_port(ctx, visiting, node_id, &"value".into(), "output.value")
        }
        "filter" => {
            let base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "filter.collection",
            )?;
            let pred = resolve_input(
                ctx.graph,
                ctx.registry,
                ctx.edges_by_target,
                node_id,
                &"predicate".into(),
            )?;
            let ResolvedInput::Wire { from } = pred else {
                return Err(ExecutionError::message(format!(
                    "filter.predicate on \"{node_id}\" must be wired"
                )));
            };
            let pred_node = from.node.clone();
            let mut out = Vec::with_capacity(base.len());
            for row in &base {
                if eval_predicate(ctx, visiting, &pred_node, Some(row))? {
                    out.push(row.clone());
                }
            }
            Ok(out)
        }
        "except" => {
            let keep = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "except.collection",
            )?;
            let exclude = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"exclude".into(),
                "except.exclude",
            )?;
            let exclude_ids: HashSet<&str> = exclude.iter().map(|r| r.id.as_str()).collect();
            Ok(keep
                .into_iter()
                .filter(|r| !exclude_ids.contains(r.id.as_str()))
                .collect())
        }
        "sort" => {
            let mut base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "sort.collection",
            )?;
            let column_name = require_string(
                &resolve_literal_or_throw(ctx, node_id, &"column".into(), "sort.column")?,
                "sort.column",
            )?;
            let direction_raw = require_string(
                &resolve_literal_or_throw(ctx, node_id, &"direction".into(), "sort.direction")?,
                "sort.direction",
            )?
            .to_lowercase();
            let sign = if direction_raw == "desc" { -1 } else { 1 };
            base.sort_by(|a, b| {
                let av = row_column(a, &column_name);
                let bv = row_column(b, &column_name);
                sort_cmp(&av, &bv, sign)
            });
            Ok(base)
        }
        "limit" => {
            let base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "limit.collection",
            )?;
            let count = require_number(
                &resolve_literal_or_throw(ctx, node_id, &"count".into(), "limit.count")?,
                "limit.count",
            )? as usize;
            Ok(base.into_iter().take(count).collect())
        }
        "offset" => {
            let base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "offset.collection",
            )?;
            let count = require_number(
                &resolve_literal_or_throw(ctx, node_id, &"count".into(), "offset.count")?,
                "offset.count",
            )? as usize;
            Ok(base.into_iter().skip(count).collect())
        }
        "project" => {
            let base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "project.collection",
            )?;
            let columns_raw = require_string(
                &resolve_literal_or_throw(ctx, node_id, &"columns".into(), "project.columns")?,
                "project.columns",
            )?;
            project_rows(&base, &columns_raw)
        }
        "group" => {
            let mut base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "group.collection",
            )?;
            let column_name = require_string(
                &resolve_literal_or_throw(ctx, node_id, &"column".into(), "group.column")?,
                "group.column",
            )?;
            let direction_raw = require_string(
                &resolve_literal_or_throw(ctx, node_id, &"direction".into(), "group.direction")?,
                "group.direction",
            )?
            .to_lowercase();
            let sign = if direction_raw == "desc" { -1 } else { 1 };
            base.sort_by(|a, b| {
                let av = row_column(a, &column_name);
                let bv = row_column(b, &column_name);
                sort_cmp(&av, &bv, sign)
            });
            Ok(base)
        }
        "search" => {
            let base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "search.collection",
            )?;
            let query = require_string(
                &resolve_literal_or_throw(ctx, node_id, &"query".into(), "search.query")?,
                "search.query",
            )?;
            ctx.host.text_search(&base, &query)
        }
        "traverse" => {
            let base = follow_collection_port(
                ctx,
                visiting,
                node_id,
                &"collection".into(),
                "traverse.collection",
            )?;
            let association = require_string(
                &resolve_literal_or_throw(
                    ctx,
                    node_id,
                    &"association".into(),
                    "traverse.association",
                )?,
                "traverse.association",
            )?;
            let direction = require_number(
                &resolve_literal_or_throw(ctx, node_id, &"direction".into(), "traverse.direction")?,
                "traverse.direction",
            )? as u8;
            if direction != 0 && direction != 1 {
                return Err(ExecutionError::message(
                    "traverse.direction must be 0 or 1".to_string(),
                ));
            }
            let edge_property_raw =
                resolve_literal_or_throw(ctx, node_id, &"edge_property".into(), "traverse.edge_property")?;
            let edge_equals_raw =
                resolve_literal_or_throw(ctx, node_id, &"edge_equals".into(), "traverse.edge_equals")?;
            let edge_property = if edge_property_raw.is_null() {
                None
            } else {
                Some(require_string(&edge_property_raw, "traverse.edge_property")?)
            };
            let edge_equals = if edge_equals_raw.is_null() {
                None
            } else {
                Some(edge_equals_raw)
            };

            let mut seen = HashSet::new();
            let mut out = Vec::new();
            for source in &base {
                let targets = ctx.host.traverse(
                    &source.id,
                    &association,
                    direction,
                    edge_property.as_deref(),
                    edge_equals.as_ref(),
                )?;
                for t in targets {
                    if seen.insert(t.id.clone()) {
                        out.push(t);
                    }
                }
            }
            Ok(out)
        }
        other if is_graph_backed_node_type(other, ctx.registry) => {
            eval_composite_collection_port(ctx, visiting, node_id, output_port)
        }
        other => Err(ExecutionError::message(format!(
            "Unsupported collection node type \"{other}\""
        ))),
    }
}

pub fn result_from_rows(rows: &[ExecutionRow]) -> crate::host::ExecutionResult {
    if rows.is_empty() {
        return crate::host::ExecutionResult {
            columns: vec!["id".into()],
            rows: vec![],
        };
    }
    let mut prop_keys = BTreeSet::new();
    for row in rows {
        for key in row.properties.keys() {
            prop_keys.insert(key.clone());
        }
    }
    let columns = std::iter::once("id".to_string())
        .chain(prop_keys.into_iter())
        .collect::<Vec<_>>();
    let out_rows = rows
        .iter()
        .map(|row| {
            let mut record = BTreeMap::new();
            record.insert("id".into(), JsonValue::String(row.id.clone()));
            for key in columns.iter().skip(1) {
                record.insert(
                    key.clone(),
                    row.properties.get(key).cloned().unwrap_or(JsonValue::Null),
                );
            }
            record
        })
        .collect();
    crate::host::ExecutionResult {
        columns,
        rows: out_rows,
    }
}
