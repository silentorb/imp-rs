//! Collection transforms library tests.

use imp_collection_transforms::collection_transforms_library;
use imp_core_types::{list_of, set_of, type_var, NodeImplementation, Type};

fn defs(id: &str) -> Vec<imp_core_types::NodeDefinition> {
    collection_transforms_library()
        .definitions
        .into_iter()
        .filter(|entry| entry.id == id)
        .collect()
}

fn def(id: &str) -> imp_core_types::NodeDefinition {
    defs(id).into_iter().next().unwrap()
}

#[test]
fn collection_transforms_library_has_expected_types() {
    let lib = collection_transforms_library();
    assert_eq!(lib.id, "imp.collection.transforms");
    assert_eq!(defs("filter").len(), 2);
    assert_eq!(def("filter").id, "filter");
    assert_eq!(
        def("filter").type_params,
        vec![imp_core_types::type_param("T")]
    );
    assert_eq!(
        def("filter").implementation,
        Some(NodeImplementation::universal("filter_list"))
    );
    assert_eq!(
        defs("filter")[1].implementation,
        Some(NodeImplementation::universal("filter_set"))
    );
    assert_eq!(
        defs("filter")[1]
            .inputs
            .get("collection")
            .map(|p| &p.ty),
        Some(&set_of(type_var("T")))
    );
    assert_eq!(defs("except").len(), 2);
    assert_eq!(defs("project").len(), 2);
    assert_eq!(defs("search").len(), 2);
    assert_eq!(defs("sort").len(), 1);
    assert_eq!(defs("limit").len(), 1);
    assert_eq!(
        def("sort").inputs.get("collection").map(|p| &p.ty),
        Some(&list_of(type_var("T")))
    );
    assert!(matches!(
        def("limit").outputs.get("collection").map(|p| &p.ty),
        Some(Type::Concrete { id, .. }) if id == "list"
    ));
    assert!(def("equals").inputs.contains_key("left"));
    assert!(lib.definitions.iter().any(|entry| entry.id == "literal"));
}
