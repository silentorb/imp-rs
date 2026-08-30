//! Registry for loading NodeLibrary catalogs.
//! Spec: imp-spec/docs/packages/imp-registry/registry.md

use imp_core_types::{NodeLibrary, NodeType, NodeTypeId};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Registry {
    pub libraries: Vec<NodeLibrary>,
    pub types: BTreeMap<NodeTypeId, NodeType>,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("NodeTypeId \"{type_id}\" is already registered (loading library \"{library_id}\")")]
pub struct DuplicateNodeTypeError {
    pub type_id: String,
    pub library_id: String,
}

pub fn create_registry() -> Registry {
    Registry {
        libraries: Vec::new(),
        types: BTreeMap::new(),
    }
}

pub fn load_library(registry: Registry, library: NodeLibrary) -> Result<Registry, DuplicateNodeTypeError> {
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

    Ok(Registry { libraries, types })
}

pub fn get_node_type<'a>(registry: &'a Registry, type_id: &str) -> Option<&'a NodeType> {
    registry.types.get(type_id)
}

pub fn list_node_types(registry: &Registry) -> Vec<&NodeType> {
    registry.types.values().collect()
}

pub fn list_libraries(registry: &Registry) -> &[NodeLibrary] {
    &registry.libraries
}
