//! Registry for loading NodeLibrary and GraphLibrary catalogs.
//! Spec: imp-spec/docs/packages/imp-registry/registry.md

use imp_core_types::{
    is_concrete, is_graph_backed_definition, is_type_var, GraphLibrary, NodeDefinition, NodeLibrary,
    NodeTypeId, Port, SignalType, TypeConstraint, TypeConstraintId, TypeConstraintLibrary,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Registry {
    pub libraries: Vec<NodeLibrary>,
    pub graph_libraries: Vec<GraphLibrary>,
    pub type_constraint_libraries: Vec<TypeConstraintLibrary>,
    /// Overload sets keyed by NodeTypeId.
    pub definitions: BTreeMap<NodeTypeId, Vec<NodeDefinition>>,
    pub type_constraints: BTreeMap<TypeConstraintId, TypeConstraint>,
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
#[error(
    "TypeConstraintId \"{constraint_id}\" is already registered (loading library \"{library_id}\")"
)]
pub struct DuplicateTypeConstraintError {
    pub constraint_id: String,
    pub library_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub struct InvalidGraphLibraryError {
    pub definition_id: String,
    pub library_id: String,
    pub message: String,
}

impl std::fmt::Display for InvalidGraphLibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GraphLibrary \"{}\" entry \"{}\": {}",
            self.library_id, self.definition_id, self.message
        )
    }
}

pub fn create_registry() -> Registry {
    Registry {
        libraries: Vec::new(),
        graph_libraries: Vec::new(),
        type_constraint_libraries: Vec::new(),
        definitions: BTreeMap::new(),
        type_constraints: BTreeMap::new(),
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

    fn type_key(signal_type: &SignalType, param_index: &BTreeMap<&str, usize>) -> String {
        if is_type_var(signal_type) {
            let SignalType::Var { param } = signal_type else {
                unreachable!()
            };
            return match param_index.get(param.as_str()) {
                Some(index) => format!("#{index}"),
                None => format!("?{param}"),
            };
        }
        if !is_concrete(signal_type) {
            return "?".into();
        }
        let SignalType::Concrete { id, args } = signal_type else {
            unreachable!()
        };
        if args.is_empty() {
            id.clone()
        } else {
            let args_key = args
                .iter()
                .map(|arg| type_key(arg, param_index))
                .collect::<Vec<_>>()
                .join(",");
            format!("{id}<{args_key}>")
        }
    }

    fn ports_key(ports: &BTreeMap<String, Port>, param_index: &BTreeMap<&str, usize>) -> String {
        let mut keys: Vec<_> = ports.keys().cloned().collect();
        keys.sort();
        keys.into_iter()
            .map(|id| {
                let port = &ports[&id];
                format!("{id}:{}", type_key(&port.signal_type, param_index))
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

pub fn load_node_library(
    registry: Registry,
    library: NodeLibrary,
) -> Result<Registry, DuplicateNodeTypeError> {
    let library_id = library.id.clone();
    let definitions = library.definitions.clone();
    let mut registry = merge_definitions(registry, &library_id, definitions)?;
    registry.libraries.push(library);
    Ok(registry)
}

pub fn load_library(
    registry: Registry,
    library: NodeLibrary,
) -> Result<Registry, DuplicateNodeTypeError> {
    load_node_library(registry, library)
}

pub fn load_graph_library(
    registry: Registry,
    library: GraphLibrary,
) -> Result<Registry, RegistryError> {
    for definition in &library.definitions {
        if !is_graph_backed_definition(definition) {
            return Err(RegistryError::InvalidGraphLibrary(InvalidGraphLibraryError {
                library_id: library.id.clone(),
                definition_id: definition.id.clone(),
                message: "must include body and bindings".into(),
            }));
        }
    }
    let mut registry = merge_definitions(registry, &library.id, library.definitions.clone())
        .map_err(RegistryError::DuplicateNodeType)?;
    registry.graph_libraries.push(library);
    Ok(registry)
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
        graph_libraries: registry.graph_libraries,
        type_constraint_libraries,
        definitions: registry.definitions,
        type_constraints,
    })
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error(transparent)]
    DuplicateNodeType(#[from] DuplicateNodeTypeError),
    #[error(transparent)]
    InvalidGraphLibrary(#[from] InvalidGraphLibraryError),
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

pub fn get_type_constraint<'a>(
    registry: &'a Registry,
    constraint_id: &str,
) -> Option<&'a TypeConstraint> {
    registry.type_constraints.get(constraint_id)
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

pub fn list_type_constraints(registry: &Registry) -> Vec<&TypeConstraint> {
    registry.type_constraints.values().collect()
}

pub fn list_libraries(registry: &Registry) -> &[NodeLibrary] {
    &registry.libraries
}

pub fn list_graph_libraries(registry: &Registry) -> &[GraphLibrary] {
    &registry.graph_libraries
}

pub fn list_type_constraint_libraries(registry: &Registry) -> &[TypeConstraintLibrary] {
    &registry.type_constraint_libraries
}

pub fn has_graph_backed_definitions(registry: &Registry) -> bool {
    list_node_definitions(registry)
        .into_iter()
        .any(is_graph_backed_definition)
}
