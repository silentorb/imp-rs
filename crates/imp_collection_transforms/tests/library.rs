//! Collection transforms library tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::{list_of, type_var, SignalType};

fn def(id: &str) -> imp_core_types::NodeDefinition {
    collection_transforms_library()
        .definitions
        .iter()
        .find(|entry| entry.id == id)
        .cloned()
        .unwrap()
}

#[test]
fn collection_transforms_library_has_expected_types() {
    let lib = collection_transforms_library();
    assert_eq!(lib.id, "imp.collection.transforms");
    assert_eq!(def("filter").id, "filter");
    assert_eq!(def("filter").type_params, vec![imp_core_types::type_param("T")]);
    assert_eq!(def("except").id, "except");
    assert_eq!(
        def("sort").inputs.get("collection").map(|p| &p.signal_type),
        Some(&list_of(type_var("T")))
    );
    assert!(matches!(
        def("limit").outputs.get("collection").map(|p| &p.signal_type),
        Some(SignalType::Concrete { id, .. }) if id == "list"
    ));
    assert!(def("equals").inputs.contains_key("left"));
    assert!(lib.definitions.iter().any(|entry| entry.id == "literal"));
}
