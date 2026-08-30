//! Dynamic Imp graph execution entry point.
//! Spec: imp-spec/docs/packages/imp-execution/execution.md

use crate::error::ExecutionError;
use crate::eval::{eval_collection_port, result_from_rows, EvalContext};
use crate::host::{
    ExecutionCapabilities, ExecutionHost, ExecutionResult, DEFAULT_EXECUTION_CAPABILITIES,
};
use crate::resolve::{default_sink, default_source, index_edges_by_target, require_node};
use imp_core_types::{Graph, PortReference};
use imp_registry::Registry;
use std::collections::BTreeSet;

pub struct ExecuteGraphOptions<'a, H: ExecutionHost> {
    pub registry: &'a Registry,
    pub host: &'a H,
    pub capabilities: ExecutionCapabilities,
    pub source: Option<PortReference>,
    pub sink: Option<PortReference>,
}

impl<'a, H: ExecutionHost> ExecuteGraphOptions<'a, H> {
    pub fn new(registry: &'a Registry, host: &'a H) -> Self {
        Self {
            registry,
            host,
            capabilities: DEFAULT_EXECUTION_CAPABILITIES,
            source: None,
            sink: None,
        }
    }
}

pub fn execute_graph<H: ExecutionHost>(
    graph: &Graph,
    options: ExecuteGraphOptions<'_, H>,
) -> Result<ExecutionResult, ExecutionError> {
    if !options.capabilities.read {
        return Err(ExecutionError::ReadCapabilityRequired);
    }
    if options.capabilities.write {
        return Err(ExecutionError::WriteCapabilityRejected);
    }

    let source = match options.source {
        Some(s) => s,
        None => default_source(graph)?,
    };
    let sink = match options.sink {
        Some(s) => s,
        None => default_sink(graph)?,
    };
    let edges_by_target = index_edges_by_target(graph)?;

    let ctx = EvalContext {
        graph,
        registry: options.registry,
        host: options.host,
        edges_by_target: &edges_by_target,
        source_node_id: source.node.clone(),
    };

    let mut visiting = BTreeSet::new();

    let sink_node = require_node(graph, &sink.node)?;
    let rows = if sink_node.node_type == "output" && sink.port == "value" {
        eval_collection_port(&ctx, &mut visiting, &sink.node, &sink.port, None)?
    } else {
        let key = (sink.node.clone(), sink.port.clone());
        let edge = edges_by_target.get(&key).ok_or_else(|| ExecutionError::UnwiredSink {
            node: sink.node.clone(),
            port: sink.port.clone(),
        })?;
        eval_collection_port(
            &ctx,
            &mut visiting,
            &edge.from.node,
            &edge.from.port,
            None,
        )?
    };

    Ok(result_from_rows(&rows))
}
