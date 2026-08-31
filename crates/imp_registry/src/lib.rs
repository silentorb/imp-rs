//! Registry for loading NodeLibrary, GraphTypeLibrary, and TypeConstraintLibrary catalogs.
//! Spec: imp-spec/docs/packages/imp-registry/registry.md

use imp_core_types::{
    GraphType, GraphTypeId, GraphTypeLibrary, NodeLibrary, NodeType, NodeTypeId,
    TypeConstraint, TypeConstraintId, TypeConstraintLibrary,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Registry {
    pub libraries: Vec<NodeLibrary>,
    pub graph_type_libraries: Vec<GraphTypeLibrary>,
    pub type_constraint_libraries: Vec<TypeConstraintLibrary>,
    pub types: BTreeMap<NodeTypeId, NodeType>,
    pub graph_types: BTreeMap<GraphTypeId, GraphType>,
    pub type_constraints: BTreeMap<TypeConstraintId, TypeConstraint>,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("NodeTypeId \"{type_id}\" is already registered (loading library \"{library_id}\")")]
pub struct DuplicateNodeTypeError {
    pub type_id: String,
    pub library_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("GraphTypeId \"{type_id}\" is already registered (loading library \"{library_id}\")")]
pub struct DuplicateGraphTypeError {
    pub type_id: String,
    pub library_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "TypeConstraintId \"{constraint_id}\" is already registered (loading library \"{library_id}\")"
)]
pub struct DuplicateTypeConstraintError {
    pub constraint_id: String,
    pub library_id: String,
}

pub fn create_registry() -> Registry {
    Registry {
        libraries: Vec::new(),
        graph_type_libraries: Vec::new(),
        type_constraint_libraries: Vec::new(),
        types: BTreeMap::new(),
        graph_types: BTreeMap::new(),
        type_constraints: BTreeMap::new(),
    }
}

pub fn load_library(
    registry: Registry,
    library: NodeLibrary,
) -> Result<Registry, DuplicateNodeTypeError> {
    let mut types = registry.types.clone();

    for (key, node_type) in &library.types {
        if types.contains_key(key) {
            return Err(DuplicateNodeTypeError {
                type_id: key.clone(),
                library_id: library.id.clone(),
            });
        }
        types.insert(key.clone(), node_type.clone());
    }

    let mut libraries = registry.libraries.clone();
    libraries.push(library);

    Ok(Registry {
        libraries,
        graph_type_libraries: registry.graph_type_libraries,
        type_constraint_libraries: registry.type_constraint_libraries,
        types,
        graph_types: registry.graph_types,
        type_constraints: registry.type_constraints,
    })
}

pub fn load_graph_type_library(
    registry: Registry,
    library: GraphTypeLibrary,
) -> Result<Registry, DuplicateGraphTypeError> {
    let mut graph_types = registry.graph_types.clone();

    for (key, graph_type) in &library.types {
        if graph_types.contains_key(key) {
            return Err(DuplicateGraphTypeError {
                type_id: key.clone(),
                library_id: library.id.clone(),
            });
        }
        graph_types.insert(key.clone(), graph_type.clone());
    }

    let mut graph_type_libraries = registry.graph_type_libraries.clone();
    graph_type_libraries.push(library);

    Ok(Registry {
        libraries: registry.libraries,
        graph_type_libraries,
        type_constraint_libraries: registry.type_constraint_libraries,
        types: registry.types,
        graph_types,
        type_constraints: registry.type_constraints,
    })
}

pub fn load_type_constraint_library(
    registry: Registry,
    library: TypeConstraintLibrary,
) -> Result<Registry, DuplicateTypeConstraintError> {
    let mut type_constraints = registry.type_constraints.clone();

    for (key, constraint) in &library.constraints {
        if type_constraints.contains_key(key) {
            return Err(DuplicateTypeConstraintError {
                constraint_id: key.clone(),
                library_id: library.id.clone(),
            });
        }
        type_constraints.insert(key.clone(), constraint.clone());
    }

    let mut type_constraint_libraries = registry.type_constraint_libraries.clone();
    type_constraint_libraries.push(library);

    Ok(Registry {
        libraries: registry.libraries,
        graph_type_libraries: registry.graph_type_libraries,
        type_constraint_libraries,
        types: registry.types,
        graph_types: registry.graph_types,
        type_constraints,
    })
}

pub fn get_node_type<'a>(registry: &'a Registry, type_id: &str) -> Option<&'a NodeType> {
    registry.types.get(type_id)
}

pub fn get_graph_type<'a>(registry: &'a Registry, type_id: &str) -> Option<&'a GraphType> {
    registry.graph_types.get(type_id)
}

pub fn get_type_constraint<'a>(
    registry: &'a Registry,
    constraint_id: &str,
) -> Option<&'a TypeConstraint> {
    registry.type_constraints.get(constraint_id)
}

pub fn list_node_types(registry: &Registry) -> Vec<&NodeType> {
    registry.types.values().collect()
}

pub fn list_graph_types(registry: &Registry) -> Vec<&GraphType> {
    registry.graph_types.values().collect()
}

pub fn list_type_constraints(registry: &Registry) -> Vec<&TypeConstraint> {
    registry.type_constraints.values().collect()
}

pub fn list_libraries(registry: &Registry) -> &[NodeLibrary] {
    &registry.libraries
}

pub fn list_graph_type_libraries(registry: &Registry) -> &[GraphTypeLibrary] {
    &registry.graph_type_libraries
}

pub fn list_type_constraint_libraries(registry: &Registry) -> &[TypeConstraintLibrary] {
    &registry.type_constraint_libraries
}
