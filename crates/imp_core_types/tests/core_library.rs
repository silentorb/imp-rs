//! Core library tests.

use imp_core_types::core_node_library;

fn def<'a>(lib: &'a imp_core_types::NodeLibrary, id: &str) -> &'a imp_core_types::NodeDefinition {
    lib.definitions
        .iter()
        .find(|entry| entry.id == id)
        .unwrap()
}

#[test]
fn core_node_library_has_boundary_definitions() {
    let lib = core_node_library();
    assert_eq!(lib.id, "imp.core");

    let input = def(&lib, "input");
    assert_eq!(input.outputs.get("value").unwrap().id, "value");

    let output = def(&lib, "output");
    assert_eq!(output.inputs.get("value").unwrap().id, "value");

    let parameter = def(&lib, "parameter");
    assert_eq!(parameter.outputs.get("value").unwrap().id, "value");
}
