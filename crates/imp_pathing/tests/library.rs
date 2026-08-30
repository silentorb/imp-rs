//! Pathing catalog tests.

use imp_core_types::PrimitiveValue;
use imp_pathing::pathing_library;
use imp_registry::{create_registry, get_node_type, load_library};

#[test]
fn has_expected_identity_and_traverse_type() {
    let lib = pathing_library();
    assert_eq!(lib.id, "imp.pathing");

    let traverse = lib.types.get("traverse").unwrap();
    assert_eq!(traverse.id, "traverse");
    assert_eq!(
        traverse.inputs.get("collection").unwrap().signal_type.id,
        "collection"
    );
    assert_eq!(
        traverse.inputs.get("association").unwrap().signal_type.id,
        "string"
    );
    assert_eq!(
        traverse.inputs.get("direction").unwrap().signal_type.id,
        "number"
    );
    assert_eq!(
        traverse.inputs.get("direction").unwrap().default_value,
        Some(PrimitiveValue::Number(0.0))
    );
    assert_eq!(
        traverse.inputs.get("edge_property").unwrap().signal_type.id,
        "string"
    );
    assert_eq!(
        traverse.inputs.get("edge_property").unwrap().default_value,
        Some(PrimitiveValue::Null)
    );
    assert_eq!(
        traverse.inputs.get("edge_equals").unwrap().signal_type.id,
        "any"
    );
    assert_eq!(
        traverse.inputs.get("edge_equals").unwrap().default_value,
        Some(PrimitiveValue::Null)
    );
    assert_eq!(
        traverse.outputs.get("collection").unwrap().signal_type.id,
        "collection"
    );
}

#[test]
fn loads_into_imp_registry_without_conflicts() {
    let registry = load_library(create_registry(), pathing_library()).unwrap();
    assert_eq!(
        get_node_type(&registry, "traverse")
            .unwrap()
            .outputs
            .get("collection")
            .unwrap()
            .signal_type
            .id,
        "collection"
    );
}
