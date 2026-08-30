//! Collection transforms catalog tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::PrimitiveValue;
use imp_registry::{create_registry, get_node_type, load_library};

#[test]
fn has_expected_identity_and_transform_types() {
    let lib = collection_transforms_library();
    assert_eq!(lib.id, "imp.collection.transforms");
    assert_eq!(lib.types.get("filter").unwrap().id, "filter");
    assert_eq!(lib.types.get("except").unwrap().id, "except");
    assert_eq!(
        lib.types
            .get("except")
            .unwrap()
            .inputs
            .get("exclude")
            .unwrap()
            .signal_type
            .id,
        "collection"
    );
    assert_eq!(
        lib.types
            .get("sort")
            .unwrap()
            .inputs
            .get("direction")
            .unwrap()
            .default_value,
        Some(PrimitiveValue::String("asc".into()))
    );
    assert_eq!(
        lib.types
            .get("group")
            .unwrap()
            .inputs
            .get("direction")
            .unwrap()
            .default_value,
        Some(PrimitiveValue::String("asc".into()))
    );
    assert_eq!(
        lib.types
            .get("group")
            .unwrap()
            .inputs
            .get("column")
            .unwrap()
            .signal_type
            .id,
        "string"
    );
    assert!(lib.types.contains_key("equals"));
    assert!(lib.types.contains_key("not_equals"));
    assert!(lib.types.contains_key("less_than"));
    assert!(lib.types.contains_key("greater_than"));
    assert!(lib.types.contains_key("column"));
    assert!(lib.types.contains_key("literal"));
}

#[test]
fn loads_into_imp_registry_without_conflicts() {
    let registry = load_library(create_registry(), collection_transforms_library()).unwrap();
    assert_eq!(
        get_node_type(&registry, "filter")
            .unwrap()
            .outputs
            .get("collection")
            .unwrap()
            .signal_type
            .id,
        "collection"
    );
    assert_eq!(
        get_node_type(&registry, "except")
            .unwrap()
            .inputs
            .get("exclude")
            .unwrap()
            .signal_type
            .id,
        "collection"
    );
    assert_eq!(
        get_node_type(&registry, "equals")
            .unwrap()
            .outputs
            .get("value")
            .unwrap()
            .signal_type
            .id,
        "boolean"
    );
}
