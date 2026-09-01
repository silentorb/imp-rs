//! Dynamic Imp graph execution entry point.
//! Spec: imp-ts/docs/features/execution.md

use crate::error::ExecutionError;
use crate::eval::{eval_collection_port, result_from_rows, EvalContext};
use crate::host::{
    ExecutionCapabilities, ExecutionHost, ExecutionResult, DEFAULT_EXECUTION_CAPABILITIES,
};
use crate::resolve::{default_sink, default_source, index_edges_by_target, require_node};
use imp_core_types::{Graph, PortReference};
use imp_graph_resolve::{build_execution_program, ExecutionProgram};
use imp_registry::{has_graph_backed_definitions, Registry};
use std::collections::BTreeSet;

pub struct ExecuteGraphOptions<'a, H: ExecutionHost> {
    pub registry: &'a Registry,
    pub host: &'a H,
    pub capabilities: ExecutionCapabilities,
    pub source: Option<PortReference>,
    pub sink: Option<PortReference>,
    pub program: Option<&'a ExecutionProgram>,
}

impl<'a, H: ExecutionHost> ExecuteGraphOptions<'a, H> {
    pub fn new(registry: &'a Registry, host: &'a H) -> Self {
        Self {
            registry,
            host,
            capabilities: DEFAULT_EXECUTION_CAPABILITIES,
            source: None,
            sink: None,
            program: None,
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

    let default_program = ExecutionProgram {
        root: graph.clone(),
        subgraphs: Default::default(),
    };
    let built_program: imp_graph_resolve::ExecutionProgram;
    let program = match options.program {
        Some(existing) => existing,
        None if has_graph_backed_definitions(options.registry) => {
            built_program = build_execution_program(graph, options.registry)
                .map_err(ExecutionError::message)?;
            &built_program
        }
        None => &default_program,
    };

    let root = &program.root;
    let source = match options.source {
        Some(s) => s,
        None => default_source(root)?,
    };
    let sink = match options.sink {
        Some(s) => s,
        None => default_sink(root)?,
    };
    let edges_by_target = index_edges_by_target(root)?;

    let ctx = EvalContext {
        graph: root,
        registry: options.registry,
        host: options.host,
        edges_by_target: &edges_by_target,
        source_node_id: source.node.clone(),
        execution_program: Some(program),
        composite: None,
    };

    let mut visiting = BTreeSet::new();

    let sink_node = require_node(root, &sink.node)?;
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
