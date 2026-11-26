//! Registry for loading node libraries and type definitions.
//! Spec: imp-spec/docs/packages/imp-registry/registry.md

use imp_core_types::{
    is_graph_backed_definition, ImpFile, NodeDefinition, NodeLibrary, NodeTypeId, Port, Type,
    TypeDefinition, TypeId,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Registry {
    pub libraries: Vec<NodeLibrary>,
    pub imp_files: Vec<ImpFile>,
    /// Overload sets keyed by NodeTypeId.
    pub definitions: BTreeMap<NodeTypeId, Vec<NodeDefinition>>,
    /// The single global type namespace: type definitions from every loaded library.
    pub type_definitions: BTreeMap<TypeId, TypeDefinition>,
    /// Set on the normalized view `imp_typecheck` derives (every definition's types expanded);
    /// hosts leave it `false`.
    #[doc(hidden)]
    pub normalized: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "NodeTypeId \"{type_id}\" already has an equivalent overload (loading library \"{library_id}\")"
)]
pub struct DuplicateNodeTypeError {
    pub type_id: String,
    pub library_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("TypeId \"{type_id}\" is already defined (loading library \"{library_id}\")")]
pub struct DuplicateTypeError {
    pub type_id: String,
    pub library_id: String,
}

pub fn create_registry() -> Registry {
    Registry {
        libraries: Vec::new(),
        imp_files: Vec::new(),
        definitions: BTreeMap::new(),
        type_definitions: BTreeMap::new(),
        normalized: false,
    }
}

/// Canonical fingerprint for overload-signature equivalence (param ids normalized by declaration order).
pub fn signature_fingerprint(definition: &NodeDefinition) -> String {
    let param_index: BTreeMap<&str, usize> = definition
        .type_params
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.as_str(), i))
        .collect();

    fn type_key(ty: &Type, param_index: &BTreeMap<&str, usize>) -> String {
        let key = |types: &[Type]| {
            types
                .iter()
                .map(|t| type_key(t, param_index))
                .collect::<Vec<_>>()
        };
        match ty {
            Type::Var { param } => match param_index.get(param.as_str()) {
                Some(index) => format!("#{index}"),
                None => format!("?{param}"),
            },
            Type::Concrete { id, args } if args.is_empty() => id.clone(),
            Type::Concrete { id, args } => format!("{id}<{}>", key(args).join(",")),
            Type::Union { union } => format!("({})", key(union).join("|")),
        }
    }

    fn ports_key(ports: &BTreeMap<String, Port>, param_index: &BTreeMap<&str, usize>) -> String {
        let mut keys: Vec<_> = ports.keys().cloned().collect();
        keys.sort();
        keys.into_iter()
            .map(|id| {
                let port = &ports[&id];
                format!("{id}:{}", type_key(&port.ty, param_index))
            })
            .collect::<Vec<_>>()
            .join(";")
    }

    format!(
        "in:{{{}}}|out:{{{}}}",
        ports_key(&definition.inputs, &param_index),
        ports_key(&definition.outputs, &param_index)
    )
}

fn merge_definitions(
    mut registry: Registry,
    library_id: &str,
    definitions: Vec<NodeDefinition>,
) -> Result<Registry, DuplicateNodeTypeError> {
    registry.normalized = false;
    for definition in definitions {
        let existing = registry
            .definitions
            .entry(definition.id.clone())
            .or_default();
        let fingerprint = signature_fingerprint(&definition);
        for prior in existing.iter() {
            if signature_fingerprint(prior) == fingerprint {
                return Err(DuplicateNodeTypeError {
                    type_id: definition.id.clone(),
                    library_id: library_id.into(),
                });
            }
        }
        existing.push(definition);
    }
    Ok(registry)
}

/// Merges type definitions into the global namespace; a duplicate `TypeId` is an error.
fn merge_types(
    mut registry: Registry,
    library_id: &str,
    types: &[TypeDefinition],
) -> Result<Registry, DuplicateTypeError> {
    registry.normalized = false;
    for definition in types {
        if registry.type_definitions.contains_key(&definition.id) {
            return Err(DuplicateTypeError {
                type_id: definition.id.clone(),
                library_id: library_id.into(),
            });
        }
        registry
            .type_definitions
            .insert(definition.id.clone(), definition.clone());
    }
    Ok(registry)
}

pub fn load_node_library(registry: Registry, library: NodeLibrary) -> Result<Registry, RegistryError> {
    let library_id = library.id.clone();
    let registry = merge_types(registry, &library_id, &library.types)?;
    let definitions = library.definitions.clone();
    let mut registry = merge_definitions(registry, &library_id, definitions)?;
    registry.libraries.push(library);
    Ok(registry)
}

pub fn load_library(registry: Registry, library: NodeLibrary) -> Result<Registry, RegistryError> {
    load_node_library(registry, library)
}

/// Merges an Imp file's `types` and `definitions` (primitive or graph-backed) into the registry.
/// Its `nodes` / `edges` are topology, not registry content (see `imp_file_graph_source`).
pub fn load_imp_file(registry: Registry, file: ImpFile) -> Result<Registry, RegistryError> {
    let label = file.id.clone().unwrap_or_else(|| "(Imp file)".into());
    let registry = merge_types(registry, &label, &file.types)?;
    let mut registry = merge_definitions(registry, &label, file.definitions.clone())?;
    registry.imp_files.push(file);
    Ok(registry)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error(transparent)]
    DuplicateNodeType(#[from] DuplicateNodeTypeError),
    #[error(transparent)]
    DuplicateType(#[from] DuplicateTypeError),
}

/// Returns the overload set for `type_id`, or an empty slice if unknown.
pub fn get_node_definitions<'a>(registry: &'a Registry, type_id: &str) -> &'a [NodeDefinition] {
    registry
        .definitions
        .get(type_id)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Sole-definition convenience: returns the definition when the overload set has length 1.
pub fn get_node_definition<'a>(
    registry: &'a Registry,
    type_id: &str,
) -> Option<&'a NodeDefinition> {
    let overloads = get_node_definitions(registry, type_id);
    if overloads.len() == 1 {
        Some(&overloads[0])
    } else {
        None
    }
}

pub fn get_node_type<'a>(registry: &'a Registry, type_id: &str) -> Option<&'a NodeDefinition> {
    get_node_definition(registry, type_id)
}

pub fn get_type_definition<'a>(registry: &'a Registry, type_id: &str) -> Option<&'a TypeDefinition> {
    registry.type_definitions.get(type_id)
}

pub fn list_node_definitions(registry: &Registry) -> Vec<&NodeDefinition> {
    registry
        .definitions
        .values()
        .flat_map(|overloads| overloads.iter())
        .collect()
}

pub fn list_node_types(registry: &Registry) -> Vec<&NodeDefinition> {
    list_node_definitions(registry)
}

pub fn list_type_definitions(registry: &Registry) -> Vec<&TypeDefinition> {
    registry.type_definitions.values().collect()
}

pub fn list_libraries(registry: &Registry) -> &[NodeLibrary] {
    &registry.libraries
}

pub fn list_imp_files(registry: &Registry) -> &[ImpFile] {
    &registry.imp_files
}

pub fn has_graph_backed_definitions(registry: &Registry) -> bool {
    list_node_definitions(registry)
        .into_iter()
        .any(is_graph_backed_definition)
}
