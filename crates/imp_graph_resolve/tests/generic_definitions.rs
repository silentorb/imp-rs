//! Flattening generic graph-backed definitions used without explicit type args.

use imp_core_types::{
    concrete_type, core_node_library, type_var, BoundaryBindings, Edge, Graph, ImpFile, Node,
    NodeDefinition, NodeLibrary, Port, PortReference, Type, TypeDefinition, TypeParam,
};
use imp_graph_resolve::flatten_graph;
use imp_registry::{
    create_registry, load_imp_file, load_library, Registry,
};
use imp_typecheck::{check_definition, check_graph, resolve_type_args};
use std::collections::BTreeMap;

fn number() -> Type {
    concrete_type("number", vec![])
}

fn port(id: &str, ty: Type) -> (String, Port) {
    (
        id.into(),
        Port {
            id: id.into(),
            ty,
            default_value: None,
            variadic: false,
        },
    )
}

fn definition(
    id: &str,
    generic: bool,
    inputs: Vec<(String, Port)>,
    outputs: Vec<(String, Port)>,
) -> NodeDefinition {
    NodeDefinition {
        id: id.into(),
        name: None,
        type_params: if generic {
            vec![TypeParam {
                id: "A".into(),
                bounds: vec![concrete_type("Scalar", vec![])],
            }]
        } else {
            vec![]
        },
        implementation: None,
        inputs: inputs.into_iter().collect(),
        outputs: outputs.into_iter().collect(),
        body: None,
        bindings: None,
    }
}

fn node(id: &str, node_type: &str) -> (String, Node) {
    (
        id.into(),
        Node {
            id: id.into(),
            title: None,
            node_type: node_type.into(),
            type_args: vec![],
            inputs: Default::default(),
        },
    )
}

fn edge(id: &str, from: (&str, &str), to: (&str, &str)) -> (String, Edge) {
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
            order: None,
        },
    )
}

/// `twice<A: Scalar>(value: A) → A` = `mix(value, value)`.
fn registry() -> (Registry, NodeDefinition) {
    let a = type_var("A");
    let primitives = NodeLibrary {
        types: Vec::new(),
        id: "example.primitives".into(),
        definitions: vec![
            definition("number_source", false, vec![], vec![port("value", number())]),
            definition(
                "string_source",
                false,
                vec![],
                vec![port("value", concrete_type("string", vec![]))],
            ),
            definition(
                "mix",
                true,
                vec![port("a", a.clone()), port("b", a.clone())],
                vec![port("value", a.clone())],
            ),
        ],
    };
    let mut twice = definition(
        "twice",
        true,
        vec![port("value", a.clone())],
        vec![port("value", a)],
    );
    twice.body = Some(Graph {
        nodes: [node("in", "input"), node("mix", "mix"), node("out", "output")].into(),
        edges: [
            edge("e1", ("in", "value"), ("mix", "a")),
            edge("e2", ("in", "value"), ("mix", "b")),
            edge("e3", ("mix", "value"), ("out", "value")),
        ]
        .into(),
    });
    twice.bindings = Some(BoundaryBindings {
        inputs: BTreeMap::from([("value".into(), "in".into())]),
        outputs: BTreeMap::from([("value".into(), "out".into())]),
    });
    let scalar = TypeDefinition {
        id: "Scalar".into(),
        params: vec![],
        ty: imp_core_types::union_of(vec![number(), concrete_type("string", vec![])]),
    };
    let registry = load_library(create_registry(), core_node_library()).expect("core");
    let primitives = NodeLibrary {
        types: vec![scalar],
        ..primitives
    };
    let registry = load_library(registry, primitives).expect("primitives");
    let registry = load_imp_file(
        registry,
        ImpFile {
            id: Some("example.graphs".into()),
            definitions: vec![twice.clone()],
            ..ImpFile::default()
        },
    )
    .expect("graphs");
    (registry, twice)
}

fn use_site(source: &str) -> Graph {
    Graph {
        nodes: [node("src", source), node("t", "twice"), node("out", "output")].into(),
        edges: [
            edge("e1", ("src", "value"), ("t", "value")),
            edge("e2", ("t", "value"), ("out", "value")),
        ]
        .into(),
    }
}

#[test]
fn generic_definition_flattens_and_infers_from_the_use_site() {
    let (registry, twice) = registry();
    assert_eq!(check_definition(&twice, &registry), vec![]);

    for (source, expected) in [
        ("number_source", number()),
        ("string_source", concrete_type("string", vec![])),
    ] {
        let graph = use_site(source);
        assert_eq!(check_graph(&graph, &registry), vec![]);
        let flat = flatten_graph(&graph, &registry).expect("flatten");
        assert_eq!(check_graph(&flat, &registry), vec![]);
        assert_eq!(
            resolve_type_args(&flat, &"t::mix".to_string(), &registry, None),
            Some(vec![expected])
        );
    }
}
