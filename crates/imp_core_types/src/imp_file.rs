//! Imp files: the JSON shape for types, definitions, nodes and edges.
//! Spec: imp-spec/docs/packages/imp-file/imp-files.md

use std::borrow::Cow;

use crate::{
    Edge, EdgeId, EdgeProvider, GraphSource, Node, NodeDefinition, NodeId, NodeProvider,
    TypeDefinition,
};
use serde::{Deserialize, Serialize};

/// An `Edge` with its identity (file edges are array elements, not map entries).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpFileEdge {
    pub id: EdgeId,
    #[serde(flatten)]
    pub edge: Edge,
}

/// Any mix of type definitions, node definitions, nodes and edges. Every section is optional.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpFile {
    /// `Some("imp")` identifies Imp content among other JSON documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Provenance (e.g. a library id); omitted when stored as a record value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<NodeDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ImpFileEdge>,
}

/// The `format` value that marks Imp content.
pub const IMP_FILE_FORMAT: &str = "imp";

/// A top-level Imp file section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpFileSection {
    Types,
    Definitions,
    Nodes,
    Edges,
}

impl ImpFileSection {
    pub const ALL: [ImpFileSection; 4] = [Self::Types, Self::Definitions, Self::Nodes, Self::Edges];

    pub fn name(self) -> &'static str {
        match self {
            Self::Types => "types",
            Self::Definitions => "definitions",
            Self::Nodes => "nodes",
            Self::Edges => "edges",
        }
    }
}

impl ImpFile {
    /// Whether `section` holds at least one element.
    pub fn has(&self, section: ImpFileSection) -> bool {
        match section {
            ImpFileSection::Types => !self.types.is_empty(),
            ImpFileSection::Definitions => !self.definitions.is_empty(),
            ImpFileSection::Nodes => !self.nodes.is_empty(),
            ImpFileSection::Edges => !self.edges.is_empty(),
        }
    }
}

impl NodeProvider for ImpFile {
    fn get_node(&self, id: &NodeId) -> Option<Cow<'_, Node>> {
        self.nodes
            .iter()
            .find(|node| &node.id == id)
            .map(Cow::Borrowed)
    }

    fn node_ids(&self) -> Option<Vec<NodeId>> {
        Some(self.nodes.iter().map(|node| node.id.clone()).collect())
    }
}

impl EdgeProvider for ImpFile {
    fn get_edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.edges
            .iter()
            .find(|edge| &edge.id == id)
            .map(|edge| &edge.edge)
    }

    fn edge_entries(&self) -> Vec<(&EdgeId, &Edge)> {
        self.edges
            .iter()
            .map(|edge| (&edge.id, &edge.edge))
            .collect()
    }
}

/// Graph source over an Imp file's `nodes` / `edges`, in element order.
pub fn imp_file_graph_source(file: &ImpFile) -> GraphSource<'_> {
    GraphSource {
        nodes: file,
        edges: file,
    }
}

impl<'a> From<&'a ImpFile> for GraphSource<'a> {
    fn from(file: &'a ImpFile) -> Self {
        imp_file_graph_source(file)
    }
}
