//! Pathing library tests.

use imp_pathing::pathing_library;

#[test]
fn traverse_uses_collection_ports() {
    let lib = pathing_library();
    let traverse = lib
        .definitions
        .iter()
        .find(|entry| entry.id == "traverse")
        .unwrap();
    assert_eq!(traverse.type_params, vec![imp_core_types::type_param("T")]);
    assert_eq!(
        traverse.inputs.get("collection").unwrap().ty,
        imp_core_types::collection_of(imp_core_types::type_var("T"))
    );
}
