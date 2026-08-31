//! JSON graph library parsing.
//! Spec: imp-spec/docs/packages/imp-core-types/graph-libraries.md

mod parse;

pub use parse::{parse_graph_library, parse_graph_library_json, GraphLibraryParseError};
