//! Registry load/lookup tests.

use imp_core_types::{concrete_type, core_node_library, NodeDefinition, NodeLibrary, Port, Ports};
use imp_registry::{
    create_registry, get_node_definition, list_libraries, list_node_definitions, load_node_library,
    DuplicateNodeTypeError,
};
use std::collections::BTreeMap;

fn source_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.source".into(),
        definitions: vec![NodeDefinition {
            id: "source".into(),
            name: None,
            type_params: vec![],
            implementation: None,
            inputs: Ports::new(),
            outputs: BTreeMap::from([(
                "out".into(),
                Port {
                    id: "out".into(),
                    signal_type: concrete_type("signal", vec![]),
                    default_value: None,
                },
            )]),
            body: None,
            bindings: None,
        }],
    }
}

fn sink_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.sink".into(),
        definitions: vec![NodeDefinition {
            id: "sink".into(),
            name: None,
            type_params: vec![],
            implementation: None,
            inputs: BTreeMap::from([(
                "in".into(),
                Port {
                    id: "in".into(),
                    signal_type: concrete_type("signal", vec![]),
                    default_value: None,
                },
            )]),
            outputs: Ports::new(),
            body: None,
            bindings: None,
        }],
    }
}

fn conflicting_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.conflict".into(),
        definitions: vec![NodeDefinition {
            id: "source".into(),
            name: None,
            type_params: vec![],
            implementation: None,
            inputs: Ports::new(),
            outputs: Ports::new(),
            body: None,
            bindings: None,
        }],
    }
}

#[test]
fn empty_registry_has_no_definitions_or_libraries() {
    let registry = create_registry();
    assert!(list_node_definitions(&registry).is_empty());
    assert!(list_libraries(&registry).is_empty());
    assert!(get_node_definition(&registry, "source").is_none());
}

#[test]
fn load_node_library_merges_definitions_and_preserves_libraries_in_order() {
    let registry = load_node_library(
        load_node_library(create_registry(), source_library()).unwrap(),
        sink_library(),
    )
    .unwrap();

    assert_eq!(get_node_definition(&registry, "source").unwrap().id, "source");
    assert_eq!(get_node_definition(&registry, "sink").unwrap().id, "sink");
    assert_eq!(list_node_definitions(&registry).len(), 2);
    assert_eq!(list_libraries(&registry).len(), 2);
}

#[test]
fn load_node_library_returns_error_on_duplicate_node_type_id() {
    let registry = load_node_library(create_registry(), source_library()).unwrap();
    let err = load_node_library(registry, conflicting_library()).unwrap_err();
    assert_eq!(
        err,
        DuplicateNodeTypeError {
            type_id: "source".into(),
            library_id: "example.conflict".into(),
        }
    );
}

#[test]
fn load_node_library_does_not_mutate_the_previous_registry() {
    let empty = create_registry();
    let with_source = load_node_library(empty.clone(), source_library()).unwrap();

    assert!(list_node_definitions(&empty).is_empty());
    assert!(list_libraries(&empty).is_empty());
    assert_eq!(list_node_definitions(&with_source).len(), 1);
}

#[test]
fn loads_core_node_library() {
    let registry = load_node_library(create_registry(), core_node_library()).unwrap();
    assert_eq!(
        get_node_definition(&registry, "input")
            .unwrap()
            .outputs
            .get("value")
            .unwrap()
            .id,
        "value"
    );
}
