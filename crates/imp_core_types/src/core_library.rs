//! Core boundary node types. Spec: imp-spec/docs/packages/imp-core-types/graph-model.md

use crate::{NodeLibrary, NodeType, Port, Ports, PrimitiveValue, SignalType};
use std::collections::BTreeMap;

fn any_signal() -> SignalType {
    SignalType {
        id: "any".into(),
    }
}

fn string_signal() -> SignalType {
    SignalType {
        id: "string".into(),
    }
}

fn port(id: &str, signal_type: SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// Core Imp boundary nodes — one instance per external port.
pub fn core_node_library() -> NodeLibrary {
    let any = any_signal();
    let string = string_signal();

    let mut types = BTreeMap::new();

    types.insert(
        "input".into(),
        NodeType {
            id: "input".into(),
            inputs: Ports::new(),
            outputs: BTreeMap::from([(
                "value".into(),
                port("value", any.clone(), None),
            )]),
        },
    );

    types.insert(
        "output".into(),
        NodeType {
            id: "output".into(),
            inputs: BTreeMap::from([(
                "value".into(),
                port("value", any.clone(), None),
            )]),
            outputs: Ports::new(),
        },
    );

    types.insert(
        "parameter".into(),
        NodeType {
            id: "parameter".into(),
            inputs: BTreeMap::from([
                (
                    "label".into(),
                    port(
                        "label",
                        string.clone(),
                        Some(PrimitiveValue::String(String::new())),
                    ),
                ),
                (
                    "value".into(),
                    port("value", any.clone(), Some(PrimitiveValue::Null)),
                ),
            ]),
            outputs: BTreeMap::from([(
                "value".into(),
                port("value", any, None),
            )]),
        },
    );

    NodeLibrary {
        id: "imp.core".into(),
        types,
    }
}
