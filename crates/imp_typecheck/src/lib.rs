//! Static signal-type checking for Imp graphs.
//! Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

mod assignable;
mod check;
mod check_definition;
mod check_type_param_bounds;
mod graph_analysis;
mod instantiate;
mod node_resolver;
mod resolve_implementation;
mod resolve_type_args;
mod subtype;
mod substitution;
mod unify;

pub use assignable::assignable;
pub use check::{check_graph, check_graph_with_resolver};
pub use check_definition::{check_definition, check_graph_definitions};
pub use check_type_param_bounds::check_type_param_bounds;
pub use graph_analysis::{
    candidate_survives, catalog_port_type, catalog_port_type_from_definition,
    collect_graph_substitution, collect_host_endpoints, effective_definition, effective_node_type,
    lookup_node, resolve_port_type, resolve_port_type_with_host, resolve_selections,
    resolve_selections_with_host, HostEndpoints, PortDirection, ResolveSelections,
    SelectedDefinitions,
};
pub use node_resolver::{NodeResolver, ResolvedEndpoint};
pub use resolve_implementation::{
    effective_node_implementation, resolve_implementation, signal_type_matches,
    ImplementationResolveError,
};
pub use resolve_type_args::resolve_type_args;
pub use substitution::TypeCheckError;
pub use instantiate::{instantiate_node_definition, instantiate_node_type};
pub use subtype::is_subtype_id;
pub use substitution::{
    apply_substitution, contains_type_var, is_any_type, scope_signal_type, scoped_param,
    Substitution,
};
pub use unify::{format_signal_type, unify, UnifyError};
