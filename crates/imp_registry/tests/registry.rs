//! Registry load/lookup tests.

use imp_core_types::core_node_library;
use imp_core_types::{NodeLibrary, NodeType, Port, Ports, SignalType};
use imp_registry::{
    create_registry, get_node_type, list_libraries, list_node_types, load_library,
    DuplicateNodeTypeError,
};
use std::collections::BTreeMap;

fn source_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.source".into(),
        types: BTreeMap::from([(
            "source".into(),
            NodeType {
                id: "source".into(),
                inputs: Ports::new(),
                outputs: BTreeMap::from([(
                    "out".into(),
                    Port {
                        id: "out".into(),
                        signal_type: SignalType {
                            id: "signal".into(),
                        },
                        default_value: None,
                    },
                )]),
            },
        )]),
    }
}

fn sink_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.sink".into(),
        types: BTreeMap::from([(
            "sink".into(),
            NodeType {
                id: "sink".into(),
                inputs: BTreeMap::from([(
                    "in".into(),
                    Port {
                        id: "in".into(),
                        signal_type: SignalType {
                            id: "signal".into(),
                        },
                        default_value: None,
                    },
                )]),
                outputs: Ports::new(),
            },
        )]),
    }
}

fn conflicting_library() -> NodeLibrary {
    NodeLibrary {
        id: "example.conflict".into(),
        types: BTreeMap::from([(
            "source".into(),
            NodeType {
                id: "source".into(),
                inputs: Ports::new(),
                outputs: Ports::new(),
            },
        )]),
    }
}

#[test]
fn empty_registry_has_no_types_or_libraries() {
    let registry = create_registry();
    assert!(list_node_types(&registry).is_empty());
    assert!(list_libraries(&registry).is_empty());
    assert!(get_node_type(&registry, "source").is_none());
}

#[test]
fn load_library_merges_types_and_preserves_libraries_in_order() {
    let registry = load_library(
        load_library(create_registry(), source_library()).unwrap(),
        sink_library(),
    )
    .unwrap();

    assert_eq!(get_node_type(&registry, "source").unwrap().id, "source");
    assert_eq!(get_node_type(&registry, "sink").unwrap().id, "sink");
    assert_eq!(list_node_types(&registry).len(), 2);
    assert_eq!(list_libraries(&registry).len(), 2);
    assert_eq!(list_libraries(&registry)[0].id, "example.source");
    assert_eq!(list_libraries(&registry)[1].id, "example.sink");
}

#[test]
fn load_library_returns_error_on_duplicate_node_type_id() {
    let registry = load_library(create_registry(), source_library()).unwrap();
    let err = load_library(registry, conflicting_library()).unwrap_err();
    assert_eq!(
        err,
        DuplicateNodeTypeError {
            type_id: "source".into(),
            library_id: "example.conflict".into(),
        }
    );
}

#[test]
fn load_library_does_not_mutate_the_previous_registry() {
    let empty = create_registry();
    let with_source = load_library(empty.clone(), source_library()).unwrap();

    assert!(list_node_types(&empty).is_empty());
    assert!(list_libraries(&empty).is_empty());
    assert_eq!(list_node_types(&with_source).len(), 1);
}

#[test]
fn loads_core_node_library() {
    let registry = load_library(create_registry(), core_node_library()).unwrap();
    assert_eq!(
        get_node_type(&registry, "input")
            .unwrap()
            .outputs
            .get("value")
            .unwrap()
            .id,
        "value"
    );
    assert_eq!(
        get_node_type(&registry, "output")
            .unwrap()
            .inputs
            .get("value")
            .unwrap()
            .id,
        "value"
    );
    assert_eq!(
        get_node_type(&registry, "parameter")
            .unwrap()
            .outputs
            .get("value")
            .unwrap()
            .id,
        "value"
    );
}
