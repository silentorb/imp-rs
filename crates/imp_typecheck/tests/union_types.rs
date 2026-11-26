//! Union types and type definitions: normalization, assignability, weak union edges, bounds,
//! dispatch and definition validation. Mirrors imp-ts `union-types.test.ts`.

use imp_core_types::{
    concrete_type, core_node_library, list_of, type_var, union_of, Edge, Graph, ImplementationCase,
    Node, NodeDefinition, NodeImplementation, NodeLibrary, Port, PortReference, Type,
    TypeDefinition, TypeParam,
};
use imp_registry::{create_registry, load_library, Registry};
use imp_typecheck::{
    assignable, check_graph, check_graph_definitions, check_type_definitions, format_type,
    normalize_type, resolve_implementation, resolve_type_args, rigid_type, unify, Substitution,
};
use std::collections::BTreeMap;

fn ty(id: &str) -> Type {
    concrete_type(id, vec![])
}

fn mono() -> Type {
    ty("audio_mono")
}

fn stereo() -> Type {
    ty("audio_stereo")
}

fn type_def(id: &str, params: &[&str], ty: Type) -> TypeDefinition {
    TypeDefinition {
        id: id.into(),
        params: params
            .iter()
            .map(|param| TypeParam {
                id: (*param).into(),
                bounds: vec![],
            })
            .collect(),
        ty,
    }
}

fn audio_types() -> Vec<TypeDefinition> {
    vec![
        type_def("Audio", &[], union_of(vec![mono(), stereo()])),
        type_def("Pair", &["T"], list_of(type_var("T"))),
    ]
}

fn ports(entries: &[(&str, Type)]) -> BTreeMap<String, Port> {
    entries
        .iter()
        .map(|(id, ty)| {
            (
                (*id).to_string(),
                Port {
                    id: (*id).into(),
                    ty: ty.clone(),
                    default_value: None,
                    variadic: false,
                },
            )
        })
        .collect()
}

fn def(id: &str, inputs: &[(&str, Type)], outputs: &[(&str, Type)]) -> NodeDefinition {
    NodeDefinition {
        id: id.into(),
        name: None,
        type_params: vec![],
        implementation: None,
        inputs: ports(inputs),
        outputs: ports(outputs),
        body: None,
        bindings: None,
    }
}

fn generic(definition: NodeDefinition) -> NodeDefinition {
    NodeDefinition {
        type_params: vec![TypeParam {
            id: "A".into(),
            bounds: vec![ty("Audio")],
        }],
        ..definition
    }
}

fn audio_library() -> NodeLibrary {
    let a = type_var("A");
    let render = NodeDefinition {
        implementation: Some(NodeImplementation::Dispatch {
            param_index: Some(0),
            cases: vec![
                ImplementationCase {
                    match_type: mono(),
                    implementation: "render_mono".into(),
                },
                ImplementationCase {
                    match_type: ty("Audio"),
                    implementation: "render_any_audio".into(),
                },
            ],
            default: None,
        }),
        ..generic(def("render", &[("audio", a.clone())], &[]))
    };
    NodeLibrary {
        id: "example.audio".into(),
        types: audio_types(),
        definitions: vec![
            def("mono_source", &[], &[("audio", mono())]),
            def("stereo_source", &[], &[("audio", stereo())]),
            def("audio_source", &[], &[("audio", ty("Audio"))]),
            def("mono_sink", &[("audio", mono())], &[]),
            def("audio_sink", &[("audio", ty("Audio"))], &[]),
            def("control_source", &[], &[("value", ty("control"))]),
            generic(def(
                "gain",
                &[("audio", a.clone()), ("amount", ty("control"))],
                &[("audio", a.clone())],
            )),
            generic(def(
                "mix",
                &[("a", a.clone()), ("b", a.clone())],
                &[("audio", a.clone())],
            )),
            render,
        ],
    }
}

fn registry_with(types: Vec<TypeDefinition>) -> Registry {
    let registry = load_library(create_registry(), core_node_library()).unwrap();
    let registry = load_library(registry, audio_library()).unwrap();
    if types.is_empty() {
        return registry;
    }
    load_library(
        registry,
        NodeLibrary {
            id: "extra".into(),
            definitions: vec![],
            types,
        },
    )
    .unwrap()
}

fn registry() -> Registry {
    registry_with(vec![])
}

fn graph(nodes: &[(&str, &str)], edges: &[(&str, &str, &str, &str)]) -> Graph {
    Graph {
        nodes: nodes
            .iter()
            .map(|(id, node_type)| {
                (
                    (*id).to_string(),
                    Node {
                        id: (*id).into(),
                        title: None,
                        node_type: (*node_type).into(),
                        type_args: vec![],
                        inputs: BTreeMap::new(),
                    },
                )
            })
            .collect(),
        edges: edges
            .iter()
            .enumerate()
            .map(|(index, (from, from_port, to, to_port))| {
                (
                    format!("e{index}"),
                    Edge {
                        from: PortReference {
                            node: (*from).into(),
                            port: (*from_port).into(),
                        },
                        to: PortReference {
                            node: (*to).into(),
                            port: (*to_port).into(),
                        },
                        order: None,
                    },
                )
            })
            .collect(),
    }
}

fn messages(graph: &Graph, registry: &Registry) -> Vec<String> {
    check_graph(graph, registry)
        .into_iter()
        .map(|error| error.message)
        .collect()
}

#[test]
fn normalize_expands_defined_names_and_generic_definitions() {
    let registry = registry();
    assert_eq!(
        normalize_type(&ty("Audio"), &registry),
        union_of(vec![mono(), stereo()])
    );
    assert_eq!(
        normalize_type(&concrete_type("Pair", vec![mono()]), &registry),
        list_of(mono())
    );
}

#[test]
fn normalize_flattens_dedupes_and_collapses_unions() {
    let registry = registry_with(vec![type_def(
        "Nested",
        &[],
        union_of(vec![ty("Audio"), union_of(vec![mono(), ty("control")])]),
    )]);
    assert_eq!(
        normalize_type(&ty("Nested"), &registry),
        union_of(vec![mono(), stereo(), ty("control")])
    );
    assert_eq!(
        normalize_type(&union_of(vec![mono(), mono()]), &registry),
        mono()
    );
    assert_eq!(
        normalize_type(&union_of(vec![mono(), ty("any")]), &registry),
        ty("any")
    );
}

#[test]
fn normalize_leaves_undefined_names_opaque_and_cyclic_names_unexpanded() {
    let registry = registry_with(vec![type_def("Loop", &[], ty("Loop"))]);
    assert_eq!(normalize_type(&ty("score"), &registry), ty("score"));
    assert_eq!(normalize_type(&ty("Loop"), &registry), ty("Loop"));
}

#[test]
fn union_assignability_table() {
    let union = union_of(vec![mono(), stereo()]);
    let empty = Substitution::new();
    assert!(assignable(&mono(), &union, &empty).is_ok());
    assert!(assignable(&ty("control"), &union, &empty).is_err());
    assert!(assignable(
        &union,
        &union_of(vec![mono(), stereo(), ty("control")]),
        &empty
    )
    .is_ok());
    assert!(assignable(&union, &mono(), &empty).is_err());
}

#[test]
fn variables_bind_to_the_whole_union_in_either_direction() {
    let union = union_of(vec![mono(), stereo()]);
    let empty = Substitution::new();
    let into = assignable(&type_var("A"), &union, &empty).unwrap();
    assert_eq!(format_type(&into["A"]), "audio_mono | audio_stereo");
    let from = assignable(&union, &type_var("A"), &empty).unwrap();
    assert_eq!(format_type(&from["A"]), "audio_mono | audio_stereo");
}

#[test]
fn different_bindings_through_different_members_are_ambiguous() {
    let target = union_of(vec![list_of(ty("string")), list_of(ty("number"))]);
    let err = assignable(&list_of(type_var("T")), &target, &Substitution::new()).unwrap_err();
    assert!(err.message.contains("ambiguous"));
}

#[test]
fn unions_are_equal_regardless_of_member_order() {
    let empty = Substitution::new();
    assert!(unify(
        &union_of(vec![mono(), stereo()]),
        &union_of(vec![stereo(), mono()]),
        &empty
    )
    .is_ok());
    assert!(unify(&union_of(vec![mono(), stereo()]), &mono(), &empty).is_err());
}

#[test]
fn a_rigid_parameter_flows_into_types_its_bounds_fit() {
    let union = union_of(vec![mono(), stereo()]);
    let rigid = rigid_type("A", std::slice::from_ref(&union));
    let empty = Substitution::new();
    assert!(assignable(&rigid, &union, &empty).is_ok());
    assert!(assignable(&rigid, &mono(), &empty).is_err());
}

#[test]
fn a_union_port_accepts_any_member() {
    let g = graph(
        &[("s", "mono_source"), ("k", "audio_sink")],
        &[("s", "audio", "k", "audio")],
    );
    assert!(messages(&g, &registry()).is_empty());
}

#[test]
fn a_union_output_cannot_feed_a_port_that_takes_one_member() {
    let g = graph(
        &[("s", "audio_source"), ("k", "mono_sink")],
        &[("s", "audio", "k", "audio")],
    );
    assert!(messages(&g, &registry())
        .iter()
        .any(|message| message.contains("audio_mono | audio_stereo")));
}

#[test]
fn an_edge_into_a_union_port_does_not_decide_an_inferred_parameter() {
    let g = graph(
        &[
            ("s", "mono_source"),
            ("c", "control_source"),
            ("g", "gain"),
            ("k", "audio_sink"),
        ],
        &[
            ("s", "audio", "g", "audio"),
            ("c", "value", "g", "amount"),
            ("g", "audio", "k", "audio"),
        ],
    );
    let registry = registry();
    assert!(messages(&g, &registry).is_empty());
    assert_eq!(
        resolve_type_args(&g, &"g".to_string(), &registry, None),
        Some(vec![mono()])
    );
}

#[test]
fn a_parameter_only_a_union_port_constrains_takes_the_whole_union() {
    let g = graph(
        &[
            ("s", "audio_source"),
            ("g", "gain"),
            ("c", "control_source"),
        ],
        &[("s", "audio", "g", "audio"), ("c", "value", "g", "amount")],
    );
    let registry = registry();
    assert!(messages(&g, &registry).is_empty());
    assert_eq!(
        resolve_type_args(&g, &"g".to_string(), &registry, None),
        Some(vec![union_of(vec![mono(), stereo()])])
    );
}

#[test]
fn bounds_by_a_named_union_reject_non_members() {
    let g = graph(
        &[
            ("c", "control_source"),
            ("d", "control_source"),
            ("m", "mix"),
        ],
        &[("c", "value", "m", "a"), ("d", "value", "m", "b")],
    );
    assert!(messages(&g, &registry())
        .iter()
        .any(|message| message.contains("does not satisfy bound audio_mono | audio_stereo")));
}

#[test]
fn mixed_layouts_into_one_parameter_still_conflict() {
    let g = graph(
        &[("s", "mono_source"), ("t", "stereo_source"), ("m", "mix")],
        &[("s", "audio", "m", "a"), ("t", "audio", "m", "b")],
    );
    assert!(!messages(&g, &registry()).is_empty());
}

#[test]
fn dispatch_patterns_match_by_assignability_including_named_unions() {
    let registry = registry();
    let mono_graph = graph(
        &[("s", "mono_source"), ("r", "render")],
        &[("s", "audio", "r", "audio")],
    );
    assert_eq!(
        resolve_implementation(&mono_graph, "r", &registry).unwrap(),
        "render_mono"
    );
    let stereo_graph = graph(
        &[("s", "stereo_source"), ("r", "render")],
        &[("s", "audio", "r", "audio")],
    );
    assert_eq!(
        resolve_implementation(&stereo_graph, "r", &registry).unwrap(),
        "render_any_audio"
    );
}

fn definition_errors(types: Vec<TypeDefinition>, definitions: Vec<NodeDefinition>) -> Vec<String> {
    let registry = load_library(
        create_registry(),
        NodeLibrary {
            id: "t".into(),
            definitions,
            types,
        },
    )
    .unwrap();
    check_type_definitions(&registry)
        .into_iter()
        .map(|error| error.message)
        .collect()
}

#[test]
fn check_type_definitions_accepts_valid_definitions() {
    assert!(definition_errors(audio_types(), vec![]).is_empty());
}

#[test]
fn check_type_definitions_reports_params_arity_and_empty_unions() {
    let mut types = audio_types();
    types.push(type_def("Free", &[], list_of(type_var("X"))));
    types.push(type_def("BadArity", &[], ty("Pair")));
    types.push(type_def("Empty", &[], union_of(vec![])));
    let errors = definition_errors(types, vec![]);
    assert!(errors
        .iter()
        .any(|m| m.contains("undeclared type parameter \"X\"")));
    assert!(errors
        .iter()
        .any(|m| m.contains("\"Pair\" expects 1 type arguments, got 0")));
    assert!(errors.iter().any(|m| m.contains("at least one member")));
}

#[test]
fn check_type_definitions_reports_direct_and_indirect_cycles() {
    let errors = definition_errors(
        vec![
            type_def("Self", &[], list_of(ty("Self"))),
            type_def("Ping", &[], union_of(vec![mono(), ty("Pong")])),
            type_def("Pong", &[], ty("Ping")),
        ],
        vec![],
    );
    for id in ["Self", "Ping", "Pong"] {
        assert!(errors
            .iter()
            .any(|m| m.contains(&format!("\"{id}\" refers to itself"))));
    }
}

#[test]
fn bad_references_in_node_definitions_are_reported_and_checked_from_graph_definitions() {
    let errors = definition_errors(audio_types(), vec![def("bad", &[("x", ty("Pair"))], &[])]);
    assert!(errors.contains(&"type \"Pair\" expects 1 type arguments, got 0".to_string()));

    let registry = load_library(
        create_registry(),
        NodeLibrary {
            id: "t".into(),
            definitions: vec![],
            types: vec![type_def("Self", &[], ty("Self"))],
        },
    )
    .unwrap();
    assert!(check_graph_definitions(&registry)
        .iter()
        .any(|error| error.type_id.as_deref() == Some("Self")));
}

#[test]
fn duplicate_type_ids_are_rejected_at_load() {
    let registry = load_library(create_registry(), audio_library()).unwrap();
    let err = load_library(
        registry,
        NodeLibrary {
            id: "dup".into(),
            definitions: vec![],
            types: vec![type_def("Audio", &[], mono())],
        },
    )
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("TypeId \"Audio\" is already defined"));
}
