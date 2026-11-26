//! Type normalization: type-definition expansion and union canonicalization.
//! Spec: imp-spec/docs/packages/imp-typecheck/type-system.md (Normalization)

use std::borrow::Cow;

use imp_core_types::{
    Graph, GraphSource, ImplementationCase, Node, NodeDefinition, NodeId, NodeImplementation,
    NodeProvider, Port, Ports, Type, TypeParam,
};
use imp_registry::{get_type_definition, Registry};

use crate::substitution::canonical_union;

/// Expands defined names (substituting their params) and canonicalizes unions: nested unions
/// flatten, `any` absorbs, duplicates drop (first occurrence wins), one-member unions collapse.
///
/// Invalid references (wrong arity, or a cycle) are left unexpanded; `check_type_definitions` and
/// the reference checks report them.
pub fn normalize_type(ty: &Type, registry: &Registry) -> Type {
    normalize_with(ty, registry, &mut Vec::new())
}

fn normalize_with(ty: &Type, registry: &Registry, expanding: &mut Vec<String>) -> Type {
    match ty {
        Type::Var { .. } => ty.clone(),
        Type::Union { union } => canonical_union(
            union
                .iter()
                .map(|member| normalize_with(member, registry, expanding))
                .collect(),
        ),
        Type::Concrete { id, args } => {
            let args: Vec<Type> = args
                .iter()
                .map(|arg| normalize_with(arg, registry, expanding))
                .collect();
            let Some(definition) = get_type_definition(registry, id) else {
                return Type::Concrete {
                    id: id.clone(),
                    args,
                };
            };
            if definition.params.len() != args.len() || expanding.contains(id) {
                return Type::Concrete {
                    id: id.clone(),
                    args,
                };
            }
            let expanded = substitute_params(&definition.ty, &definition.params, &args);
            expanding.push(id.clone());
            let normalized = normalize_with(&expanded, registry, expanding);
            expanding.pop();
            normalized
        }
    }
}

fn substitute_params(ty: &Type, params: &[TypeParam], args: &[Type]) -> Type {
    match ty {
        Type::Var { param } => params
            .iter()
            .position(|candidate| &candidate.id == param)
            .map(|index| args[index].clone())
            .unwrap_or_else(|| ty.clone()),
        other => other.map_children(|child| substitute_params(child, params, args)),
    }
}

fn normalize_ports(ports: &Ports, registry: &Registry) -> Ports {
    ports
        .iter()
        .map(|(key, port)| {
            (
                key.clone(),
                Port {
                    ty: normalize_type(&port.ty, registry),
                    ..port.clone()
                },
            )
        })
        .collect()
}

fn normalize_implementation(
    implementation: &Option<NodeImplementation>,
    registry: &Registry,
) -> Option<NodeImplementation> {
    match implementation {
        Some(NodeImplementation::Dispatch {
            param_index,
            cases,
            default,
        }) => Some(NodeImplementation::Dispatch {
            param_index: *param_index,
            cases: cases
                .iter()
                .map(|case| ImplementationCase {
                    match_type: normalize_type(&case.match_type, registry),
                    implementation: case.implementation.clone(),
                })
                .collect(),
            default: default.clone(),
        }),
        other => other.clone(),
    }
}

/// A node with its `type_args` normalized.
pub fn normalize_node(node: &Node, registry: &Registry) -> Node {
    Node {
        type_args: node
            .type_args
            .iter()
            .map(|arg| normalize_type(arg, registry))
            .collect(),
        ..node.clone()
    }
}

fn normalize_graph(graph: &Graph, registry: &Registry) -> Graph {
    Graph {
        nodes: graph
            .nodes
            .iter()
            .map(|(id, node)| (id.clone(), normalize_node(node, registry)))
            .collect(),
        edges: graph.edges.clone(),
    }
}

/// A definition with every type it carries normalized (ports, bounds, dispatch patterns, body).
pub fn normalize_definition(definition: &NodeDefinition, registry: &Registry) -> NodeDefinition {
    NodeDefinition {
        type_params: definition
            .type_params
            .iter()
            .map(|param| TypeParam {
                id: param.id.clone(),
                bounds: param
                    .bounds
                    .iter()
                    .map(|bound| normalize_type(bound, registry))
                    .collect(),
            })
            .collect(),
        implementation: normalize_implementation(&definition.implementation, registry),
        inputs: normalize_ports(&definition.inputs, registry),
        outputs: normalize_ports(&definition.outputs, registry),
        body: definition
            .body
            .as_ref()
            .map(|body| normalize_graph(body, registry)),
        ..definition.clone()
    }
}

/// The registry with every node definition normalized. Typecheck entry points run on this view,
/// so internal rules only ever see normalized types. Borrowed when already normalized.
pub fn normalized_registry(registry: &Registry) -> Cow<'_, Registry> {
    if registry.normalized {
        return Cow::Borrowed(registry);
    }
    let definitions = registry
        .definitions
        .iter()
        .map(|(id, overloads)| {
            (
                id.clone(),
                overloads
                    .iter()
                    .map(|definition| normalize_definition(definition, registry))
                    .collect(),
            )
        })
        .collect();
    Cow::Owned(Registry {
        definitions,
        normalized: true,
        ..registry.clone()
    })
}

/// Node provider whose nodes' `type_args` come back normalized.
struct NormalizedNodes<'a> {
    inner: &'a dyn NodeProvider,
    registry: &'a Registry,
}

impl NodeProvider for NormalizedNodes<'_> {
    fn get_node(&self, id: &NodeId) -> Option<Cow<'_, Node>> {
        let node = self.inner.get_node(id)?;
        if node.type_args.is_empty() {
            return Some(node);
        }
        Some(Cow::Owned(normalize_node(&node, self.registry)))
    }

    fn node_ids(&self) -> Option<Vec<NodeId>> {
        self.inner.node_ids()
    }
}

/// Runs `run` on the normalized registry and a graph source whose node `type_args` are normalized.
pub(crate) fn with_prepared<R>(
    source: GraphSource<'_>,
    registry: &Registry,
    run: impl FnOnce(GraphSource<'_>, &Registry) -> R,
) -> R {
    let registry = normalized_registry(registry);
    let nodes = NormalizedNodes {
        inner: source.nodes,
        registry: &registry,
    };
    run(
        GraphSource {
            nodes: &nodes,
            edges: source.edges,
        },
        &registry,
    )
}
