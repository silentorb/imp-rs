//! Static type checking for Imp graphs.
//! Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

mod assignable;
mod check;
mod check_definition;
mod check_type_definitions;
mod check_type_param_bounds;
mod graph_analysis;
mod instantiate;
mod normalize;
mod resolve_implementation;
mod resolve_type_args;
mod subtype;
mod substitution;
mod unify;

pub use assignable::assignable;
pub use check::check_graph;
pub use check_definition::{check_definition, check_graph_definitions};
pub use check_type_definitions::{check_type_definitions, type_reference_problems};
pub use check_type_param_bounds::check_type_param_bounds;
pub use normalize::{normalize_definition, normalize_node, normalize_type, normalized_registry};
pub use graph_analysis::{
    candidate_survives, catalog_port_type, catalog_port_type_from_definition,
    collect_graph_substitution, effective_definition, effective_node_type, resolve_port_type,
    resolve_selections, PortDirection, ResolveSelections, SelectedDefinitions,
};
pub use resolve_implementation::{
    effective_node_implementation, resolve_implementation, type_matches,
    ImplementationResolveError,
};
pub use resolve_type_args::resolve_type_args;
pub use substitution::TypeCheckError;
pub use instantiate::{instantiate_node_definition, instantiate_node_type};
pub use subtype::is_subtype_id;
pub use substitution::{
    apply_substitution, canonical_union, contains_type_var, is_any_type, rigid_type, scope_type,
    scoped_param, Substitution,
};
pub use unify::{format_type, unify, UnifyError};
