//! Core boundary `NodeLibrary` catalog tests.

use imp_core_types::{core_node_library, PrimitiveValue};

#[test]
fn has_expected_boundary_types_and_port_templates() {
    let lib = core_node_library();
    assert_eq!(lib.id, "imp.core");

    let input = lib.types.get("input").unwrap();
    assert_eq!(input.id, "input");
    assert!(input.inputs.is_empty());
    assert_eq!(input.outputs.get("value").unwrap().id, "value");
    assert_eq!(input.outputs.get("value").unwrap().signal_type.id, "any");

    let output = lib.types.get("output").unwrap();
    assert_eq!(output.id, "output");
    assert_eq!(output.inputs.get("value").unwrap().id, "value");
    assert!(output.outputs.is_empty());

    let parameter = lib.types.get("parameter").unwrap();
    assert_eq!(parameter.id, "parameter");
    let label = parameter.inputs.get("label").unwrap();
    assert_eq!(label.signal_type.id, "string");
    assert_eq!(
        label.default_value,
        Some(PrimitiveValue::String(String::new()))
    );
    let value_in = parameter.inputs.get("value").unwrap();
    assert_eq!(value_in.default_value, Some(PrimitiveValue::Null));
    assert_eq!(parameter.outputs.get("value").unwrap().signal_type.id, "any");
}
