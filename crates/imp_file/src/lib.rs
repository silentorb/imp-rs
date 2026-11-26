//! Imp file parsing, validation and serialization.
//! Spec: imp-spec/docs/packages/imp-file/imp-files.md

mod file;
mod parse;

pub use file::{
    assert_imp_file_sections, imp_file_from_graph, imp_file_graph, serialize_imp_file,
    SectionNotAllowed,
};
pub use imp_core_types::imp_file_graph_source;
pub use parse::{parse_imp_file, parse_imp_file_json, ImpFileParseError};
