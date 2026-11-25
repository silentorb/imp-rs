//! Generic inference: parameter agreement, `any` targets, variadic instantiation, conflict
//! messages, and generic graph-backed definitions.

use imp_core_types::{
    concrete_type, core_node_library, type_var, BoundaryBindings, Edge, Graph, Node,
    NodeDefinition, NodeLibrary, Port, PortReference, SignalType, TypeConstraint,
    TypeConstraintLibrary, TypeParam,
};
use imp_registry::{create_registry, load_library, load_type_constraint_library, Registry};
use imp_typecheck::{check_definition, check_graph, resolve_type_args};
use std::collections::BTreeMap;

fn number() -> SignalType {
    concrete_type("number", vec![])
}

fn string() -> SignalType {
    concrete_type("string", vec![])
}

fn port(id: &str, signal_type: SignalType, variadic: bool) -> (String, Port) {
    (
        id.into(),
        Port {
            id: id.into(),
            signal_type,
            default_value: None,
            variadic,
        },
    )
}

fn definition(
    id: &str,
    bound: Option<&str>,
    inputs: Vec<(String, Port)>,
    outputs: Vec<(String, Port)>,
) -> NodeDefinition {
    NodeDefinition {
        id: id.into(),
        name: None,
        type_params: bound
            .map(|bound| {
                vec![TypeParam {
                    id: "A".into(),
                    bounds: vec![bound.into()],
                }]
            })
            .unwrap_or_default(),
        implementation: None,
        inputs: inputs.into_iter().collect(),
        outputs: outputs.into_iter().collect(),
        body: None,
        bindings: None,
    }
}

/// `Scalar = number | string`; `Numeric = number`.
fn registry() -> Registry {
    let a = type_var("A");
    let library = NodeLibrary {
        id: "example.generic".into(),
        definitions: vec![
            definition("number_source", None, vec![], vec![port("value", number(), false)]),
            definition("string_source", None, vec![], vec![port("value", string(), false)]),
            definition(
                "mix",
                Some("Scalar"),
                vec![port("a", a.clone(), false), port("b", a.clone(), false)],
                vec![port("value", a.clone(), false)],
            ),
            definition(
                "sum",
                Some("Scalar"),
                vec![port("inputs", a.clone(), true)],
                vec![port("value", a.clone(), false)],
            ),
            definition(
                "numeric_only",
                Some("Numeric"),
                vec![port("value", a.clone(), false)],
                vec![port("value", a.clone(), false)],
            ),
            definition(
                "seeded",
                None,
                vec![port("seed", concrete_type("any", vec![]), false)],
                vec![port("value", number(), false)],
            ),
        ],
    };
    let constraints = TypeConstraintLibrary {
        id: "example.constraints".into(),
        constraints: BTreeMap::from([
            (
                "Scalar".into(),
                TypeConstraint {
                    id: "Scalar".into(),
                    members: vec![number(), string()],
                },
            ),
            (
                "Numeric".into(),
                TypeConstraint {
                    id: "Numeric".into(),
                    members: vec![number()],
                },
            ),
        ]),
    };
    let registry = load_library(create_registry(), core_node_library()).expect("core");
    let registry = load_type_constraint_library(registry, constraints).expect("constraints");
    load_library(registry, library).expect("library")
}

fn node(id: &str, node_type: &str, type_args: Vec<SignalType>) -> (String, Node) {
    (
        id.into(),
        Node {
            id: id.into(),
            title: None,
            node_type: node_type.into(),
            type_args,
            inputs: Default::default(),
        },
    )
}

fn edge(id: &str, from: (&str, &str), to: (&str, &str), order: Option<i64>) -> (String, Edge) {
    (
        id.into(),
        Edge {
            from: PortReference {
                node: from.0.into(),
                port: from.1.into(),
            },
            to: PortReference {
                node: to.0.into(),
                port: to.1.into(),
            },
            order,
        },
    )
}

fn graph(nodes: Vec<(String, Node)>, edges: Vec<(String, Edge)>) -> Graph {
    Graph {
        nodes: nodes.into_iter().collect(),
        edges: edges.into_iter().collect(),
    }
}

fn messages(graph: &Graph, registry: &Registry) -> Vec<String> {
    check_graph(graph, registry)
        .into_iter()
        .map(|error| error.message)
        .collect()
}

fn mix_graph(a_source: &str, b_source: &str) -> Graph {
    graph(
        vec![
            node("x", a_source, vec![]),
            node("y", b_source, vec![]),
            node("mix", "mix", vec![]),
            node("out", "output", vec![]),
        ],
        vec![
            edge("e1", ("x", "value"), ("mix", "a"), None),
            edge("e2", ("y", "value"), ("mix", "b"), None),
            edge("e3", ("mix", "value"), ("out", "value"), None),
        ],
    )
}

#[test]
fn matching_inputs_resolve_the_parameter() {
    let registry = registry();
    for (source, expected) in [("number_source", number()), ("string_source", string())] {
        let graph = mix_graph(source, source);
        assert!(messages(&graph, &registry).is_empty());
        assert_eq!(
            resolve_type_args(&graph, &"mix".to_string(), &registry, None),
            Some(vec![expected])
        );
    }
}

#[test]
fn mismatched_inputs_report_a_type_parameter_conflict() {
    let registry = registry();
    let errors = messages(&mix_graph("number_source", "string_source"), &registry);
    assert_eq!(
        errors,
        vec!["type parameter A is number (from input a) but input b receives string".to_string()]
    );
}

#[test]
fn mismatched_variadic_inputs_report_a_type_parameter_conflict() {
    let registry = registry();
    let graph = graph(
        vec![
            node("x", "number_source", vec![]),
            node("y", "string_source", vec![]),
            node("sum", "sum", vec![]),
            node("out", "output", vec![]),
        ],
        vec![
            edge("e1", ("x", "value"), ("sum", "inputs"), Some(0)),
            edge("e2", ("y", "value"), ("sum", "inputs"), Some(1)),
            edge("e3", ("sum", "value"), ("out", "value"), None),
        ],
    );
    let errors = messages(&graph, &registry);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].starts_with("type parameter A is "), "{errors:?}");
}

#[test]
fn any_target_does_not_decide_a_parameter() {
    let registry = registry();
    // The `any` edge id sorts first so it would be applied first without deferral.
    let mut graph = mix_graph("number_source", "string_source");
    graph
        .nodes
        .extend([node("noise", "seeded", vec![])]);
    graph
        .edges
        .extend([edge("a0", ("mix", "value"), ("noise", "seed"), None)]);
    let errors = messages(&graph, &registry);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("type parameter A"), "{errors:?}");

    let graph = {
        let mut graph = mix_graph("number_source", "number_source");
        graph.nodes.extend([node("noise", "seeded", vec![])]);
        graph
            .edges
            .extend([edge("a0", ("mix", "value"), ("noise", "seed"), None)]);
        graph
    };
    assert!(messages(&graph, &registry).is_empty());
    assert_eq!(
        resolve_type_args(&graph, &"mix".to_string(), &registry, None),
        Some(vec![number()])
    );
}

#[test]
fn explicit_type_args_keep_variadic_ports() {
    let registry = registry();
    let graph = graph(
        vec![
            node("x", "number_source", vec![]),
            node("y", "number_source", vec![]),
            node("sum", "sum", vec![number()]),
            node("out", "output", vec![]),
        ],
        vec![
            edge("e1", ("x", "value"), ("sum", "inputs"), Some(0)),
            edge("e2", ("y", "value"), ("sum", "inputs"), Some(1)),
            edge("e3", ("sum", "value"), ("out", "value"), None),
        ],
    );
    assert_eq!(messages(&graph, &registry), Vec::<String>::new());
}

/// `chain<A: bound>(a: A, b: A) → A` whose body runs `inner` on `mix(a, b)`.
fn generic_chain(bound: &str, inner: &str, inner_port: &str) -> NodeDefinition {
    let a = type_var("A");
    let mut chain = definition(
        "chain",
        Some(bound),
        vec![port("a", a.clone(), false), port("b", a.clone(), false)],
        vec![port("value", a, false)],
    );
    chain.body = Some(graph(
        vec![
            node("in_a", "input", vec![]),
            node("in_b", "input", vec![]),
            node("mix", "mix", vec![]),
            node("inner", inner, vec![]),
            node("out", "output", vec![]),
        ],
        vec![
            edge("e1", ("in_a", "value"), ("mix", "a"), None),
            edge("e2", ("in_b", "value"), ("mix", "b"), None),
            edge("e3", ("mix", "value"), ("inner", inner_port), Some(0)),
            edge("e4", ("inner", "value"), ("out", "value"), None),
        ],
    ));
    chain.bindings = Some(BoundaryBindings {
        inputs: BTreeMap::from([("a".into(), "in_a".into()), ("b".into(), "in_b".into())]),
        outputs: BTreeMap::from([("value".into(), "out".into())]),
    });
    chain
}

#[test]
fn generic_definition_body_checks_with_rigid_parameters() {
    let registry = registry();
    assert_eq!(check_definition(&generic_chain("Scalar", "sum", "inputs"), &registry), vec![]);
}

#[test]
fn generic_definition_rejects_concrete_use_of_a_parameter() {
    let registry = registry();
    let mut chain = generic_chain("Scalar", "sum", "inputs");
    let body = chain.body.as_mut().unwrap();
    body.nodes.extend([node("noise", "seeded", vec![])]);
    // A concrete `number` into the generic output: `A` is rigid, so this must fail.
    body.edges.insert(
        "e4".into(),
        Edge {
            from: PortReference {
                node: "noise".into(),
                port: "value".into(),
            },
            to: PortReference {
                node: "out".into(),
                port: "value".into(),
            },
            order: None,
        },
    );
    let errors = check_definition(&chain, &registry);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("number") && error.message.contains('A')),
        "{errors:?}"
    );
}

#[test]
fn generic_definition_bounds_must_imply_inner_bounds() {
    let registry = registry();
    let mut narrow = generic_chain("Numeric", "numeric_only", "value");
    narrow.id = "narrow".into();
    assert_eq!(check_definition(&narrow, &registry), vec![]);

    let wide = generic_chain("Scalar", "numeric_only", "value");
    let errors = check_definition(&wide, &registry);
    assert!(
        errors
            .iter()
            .any(|error| error.message == "type argument A does not satisfy bound \"Numeric\""),
        "{errors:?}"
    );
}

#[test]
fn every_node_of_a_long_generic_chain_resolves() {
    let registry = registry();
    let mut nodes = vec![node("src", "number_source", vec![]), node("out", "output", vec![])];
    let mut edges = Vec::new();
    let mut previous = "src".to_string();
    for index in 0..12 {
        let id = format!("n{index}");
        nodes.push(node(&id, "numeric_only", vec![]));
        edges.push(edge(
            &format!("e{index}"),
            (&previous, "value"),
            (&id, "value"),
            None,
        ));
        previous = id;
    }
    edges.push(edge("last", (&previous, "value"), ("out", "value"), None));
    let graph = graph(nodes, edges);
    assert_eq!(messages(&graph, &registry), Vec::<String>::new());
    for index in 0..12 {
        assert_eq!(
            resolve_type_args(&graph, &format!("n{index}"), &registry, None),
            Some(vec![number()]),
            "n{index}"
        );
    }
}
