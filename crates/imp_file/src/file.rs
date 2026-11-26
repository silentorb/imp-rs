//! Imp file serialization, graph conversion and section checks.

use imp_core_types::{Graph, ImpFile, ImpFileEdge, ImpFileSection};
use thiserror::Error;

/// JSON text with sections in canonical order (`format`, `id`, `types`, `definitions`, `nodes`,
/// `edges`); empty sections omitted; element order preserved.
pub fn serialize_imp_file(file: &ImpFile) -> Result<String, serde_json::Error> {
    serde_json::to_string(file)
}

/// The file's nodes and edges as an in-memory `Graph` (id-keyed maps).
pub fn imp_file_graph(file: &ImpFile) -> Graph {
    Graph {
        nodes: file
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect(),
        edges: file
            .edges
            .iter()
            .map(|edge| (edge.id.clone(), edge.edge.clone()))
            .collect(),
    }
}

/// An Imp file holding a graph's nodes and edges (edge ids from the graph's edge keys).
pub fn imp_file_from_graph(graph: &Graph) -> ImpFile {
    ImpFile {
        nodes: graph.nodes.values().cloned().collect(),
        edges: graph
            .edges
            .iter()
            .map(|(id, edge)| ImpFileEdge {
                id: id.clone(),
                edge: edge.clone(),
            })
            .collect(),
        ..ImpFile::default()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Imp file section \"{section}\" is not allowed here (allowed: {allowed})")]
pub struct SectionNotAllowed {
    pub section: &'static str,
    pub allowed: String,
}

/// Errors when a section outside `allowed` is non-empty (e.g. topology-only record slots).
pub fn assert_imp_file_sections(
    file: &ImpFile,
    allowed: &[ImpFileSection],
) -> Result<(), SectionNotAllowed> {
    for section in ImpFileSection::ALL {
        if !allowed.contains(&section) && file.has(section) {
            let names: Vec<&str> = allowed.iter().map(|s| s.name()).collect();
            return Err(SectionNotAllowed {
                section: section.name(),
                allowed: if names.is_empty() {
                    "none".into()
                } else {
                    names.join(", ")
                },
            });
        }
    }
    Ok(())
}
