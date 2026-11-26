//! Type-definition validation.
//! Spec: imp-spec/docs/packages/imp-typecheck/type-system.md (checkTypeDefinitions)

use std::collections::{BTreeMap, BTreeSet};

use imp_core_types::{NodeDefinition, NodeImplementation, Type, TypeDefinition};
use imp_registry::{get_type_definition, list_node_definitions, list_type_definitions, Registry};

use crate::substitution::TypeCheckError;

/// Problems with references to defined names inside `ty`: wrong argument counts and empty
/// unions. Type variables are not checked here.
pub fn type_reference_problems(ty: &Type, registry: &Registry) -> Vec<String> {
    let mut problems = Vec::new();
    collect_reference_problems(ty, registry, &mut problems);
    problems
}

fn collect_reference_problems(ty: &Type, registry: &Registry, problems: &mut Vec<String>) {
    match ty {
        Type::Union { union } if union.is_empty() => {
            problems.push("a union must have at least one member".into());
        }
        Type::Concrete { id, args } => {
            if let Some(definition) = get_type_definition(registry, id) {
                let expected = definition.params.len();
                if expected != args.len() {
                    problems.push(format!(
                        "type \"{id}\" expects {expected} type arguments, got {}",
                        args.len()
                    ));
                }
            }
        }
        _ => {}
    }
    for child in ty.children() {
        collect_reference_problems(child, registry, problems);
    }
}

fn referenced_definitions(ty: &Type, registry: &Registry, into: &mut BTreeSet<String>) {
    if let Type::Concrete { id, .. } = ty {
        if get_type_definition(registry, id).is_some() {
            into.insert(id.clone());
        }
    }
    for child in ty.children() {
        referenced_definitions(child, registry, into);
    }
}

fn unknown_params(definition: &TypeDefinition) -> Vec<String> {
    fn visit(ty: &Type, declared: &BTreeSet<&str>, unknown: &mut Vec<String>) {
        if let Type::Var { param } = ty {
            if !declared.contains(param.as_str()) && !unknown.contains(param) {
                unknown.push(param.clone());
            }
        }
        for child in ty.children() {
            visit(child, declared, unknown);
        }
    }
    let declared: BTreeSet<&str> = definition
        .params
        .iter()
        .map(|param| param.id.as_str())
        .collect();
    let mut unknown = Vec::new();
    visit(&definition.ty, &declared, &mut unknown);
    unknown
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    Visiting,
    Done,
}

/// Ids of type definitions that refer to themselves, directly or through other definitions,
/// each with the cycle it lies on.
fn cyclic_definitions(registry: &Registry) -> BTreeMap<String, Vec<String>> {
    fn visit(
        id: &str,
        registry: &Registry,
        state: &mut BTreeMap<String, Visit>,
        path: &mut Vec<String>,
        cycles: &mut BTreeMap<String, Vec<String>>,
    ) {
        match state.get(id) {
            Some(Visit::Done) => return,
            Some(Visit::Visiting) => {
                let start = path.iter().position(|entry| entry == id).unwrap_or(0);
                let mut cycle: Vec<String> = path[start..].to_vec();
                cycle.push(id.to_string());
                for member in &cycle[..cycle.len() - 1] {
                    cycles
                        .entry(member.clone())
                        .or_insert_with(|| cycle.clone());
                }
                return;
            }
            None => {}
        }
        let Some(definition) = get_type_definition(registry, id) else {
            return;
        };
        state.insert(id.to_string(), Visit::Visiting);
        path.push(id.to_string());
        let mut refs = BTreeSet::new();
        referenced_definitions(&definition.ty, registry, &mut refs);
        for reference in &refs {
            visit(reference, registry, state, path, cycles);
        }
        path.pop();
        state.insert(id.to_string(), Visit::Done);
    }

    let mut state = BTreeMap::new();
    let mut cycles = BTreeMap::new();
    for definition in list_type_definitions(registry) {
        visit(
            &definition.id,
            registry,
            &mut state,
            &mut Vec::new(),
            &mut cycles,
        );
    }
    cycles
}

fn definition_types(definition: &NodeDefinition) -> Vec<&Type> {
    let mut types: Vec<&Type> = definition
        .inputs
        .values()
        .chain(definition.outputs.values())
        .map(|port| &port.ty)
        .collect();
    for param in &definition.type_params {
        types.extend(param.bounds.iter());
    }
    if let Some(NodeImplementation::Dispatch { cases, .. }) = &definition.implementation {
        types.extend(cases.iter().map(|case| &case.match_type));
    }
    if let Some(body) = &definition.body {
        for node in body.nodes.values() {
            types.extend(node.type_args.iter());
        }
    }
    types
}

/// Validates every registered type definition (params, arity, empty unions, cycles) and the
/// references to defined names in every node definition. Run on the raw registry.
pub fn check_type_definitions(registry: &Registry) -> Vec<TypeCheckError> {
    let mut errors = Vec::new();

    for definition in list_type_definitions(registry) {
        for param in unknown_params(definition) {
            errors.push(TypeCheckError::for_type(
                definition.id.clone(),
                format!(
                    "type \"{}\" refers to undeclared type parameter \"{param}\"",
                    definition.id
                ),
            ));
        }
        for problem in type_reference_problems(&definition.ty, registry) {
            errors.push(TypeCheckError::for_type(
                definition.id.clone(),
                format!("type \"{}\": {problem}", definition.id),
            ));
        }
    }

    for (id, cycle) in cyclic_definitions(registry) {
        errors.push(TypeCheckError::for_type(
            id.clone(),
            format!(
                "type \"{id}\" refers to itself ({}); recursive types are not supported",
                cycle.join(" → ")
            ),
        ));
    }

    for definition in list_node_definitions(registry) {
        let mut seen = BTreeSet::new();
        for ty in definition_types(definition) {
            for problem in type_reference_problems(ty, registry) {
                if seen.insert(problem.clone()) {
                    errors.push(TypeCheckError::for_definition(
                        definition.id.clone(),
                        problem,
                    ));
                }
            }
        }
    }

    errors
}
