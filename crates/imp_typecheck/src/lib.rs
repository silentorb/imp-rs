//! Static signal-type checking for Imp graphs.
//! Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

mod check;
mod check_definition;
mod check_type_param_bounds;
mod graph_analysis;
mod instantiate;
mod resolve_implementation;
mod resolve_type_args;
mod substitution;
mod unify;

pub use check::check_graph;
pub use check_definition::{check_definition, check_graph_definitions};
pub use check_type_param_bounds::check_type_param_bounds;
pub use graph_analysis::resolve_port_type;
pub use resolve_implementation::{
    effective_node_implementation, resolve_implementation, signal_type_matches,
    ImplementationResolveError,
};
pub use resolve_type_args::resolve_type_args;
pub use substitution::TypeCheckError;
pub use instantiate::{instantiate_node_definition, instantiate_node_type};
pub use substitution::{
    apply_substitution, contains_type_var, is_any_type, scope_signal_type, scoped_param,
    Substitution,
};
pub use unify::{format_signal_type, unify, UnifyError};
