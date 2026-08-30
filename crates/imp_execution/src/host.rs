//! Read-only execution host and sandbox capabilities.
//! Spec: imp-spec/docs/packages/imp-execution/execution.md

use crate::error::ExecutionError;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// One row returned by an execution host.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionRow {
    pub id: String,
    pub properties: BTreeMap<String, JsonValue>,
    pub is_archived: Option<bool>,
}

impl ExecutionRow {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            properties: BTreeMap::new(),
            is_archived: None,
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

/// Default-deny extension point for future effectful operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionCapabilities {
    pub read: bool,
    pub write: bool,
}

pub const DEFAULT_EXECUTION_CAPABILITIES: ExecutionCapabilities = ExecutionCapabilities {
    read: true,
    write: false,
};

/// Tabular result from `execute_graph`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub columns: Vec<String>,
    pub rows: Vec<BTreeMap<String, JsonValue>>,
}

/// Read-only data source for dynamic Imp graph execution.
pub trait ExecutionHost {
    fn list_input_rows(&self) -> Result<Vec<ExecutionRow>, ExecutionError>;

    fn traverse(
        &self,
        source_id: &str,
        association: &str,
        direction: u8,
        edge_property: Option<&str>,
        edge_equals: Option<&JsonValue>,
    ) -> Result<Vec<ExecutionRow>, ExecutionError>;

    fn text_search(
        &self,
        _rows: &[ExecutionRow],
        _query: &str,
    ) -> Result<Vec<ExecutionRow>, ExecutionError> {
        Err(ExecutionError::MissingTextSearch)
    }
}
