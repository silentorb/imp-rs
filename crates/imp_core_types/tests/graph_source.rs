//! Graph source providers. Mirrors imp-ts `graph-source.test.ts`.

use imp_core_types::{
    graph_node_ids, graph_source, inbound_edges, materialize_graph, resolved_nodes,
    unresolved_node_ids, Edge, Graph, GraphSource, LayeredNodes, Node, NodeId, NodeProvider,
    PortReference,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

fn node(id: &str, node_type: &str) -> Node {
    Node {
        id: id.into(),
        title: None,
        node_type: node_type.into(),
        type_args: vec![],
        inputs: Default::default(),
    }
}

fn edge(from: &str, to: &str, order: i64) -> Edge {
    Edge {
        from: PortReference {
            node: from.into(),
            port: "value".into(),
        },
        to: PortReference {
            node: to.into(),
            port: "items".into(),
        },
        order: Some(order),
    }
}

fn doc() -> Graph {
    Graph {
        nodes: [
            ("a".into(), node("a", "literal")),
            ("lone".into(), node("lone", "literal")),
        ]
        .into(),
        edges: [
            ("e1".into(), edge("r", "p", 1)),
            ("e2".into(), edge("a", "p", 2)),
        ]
        .into(),
    }
}

/// Non-enumerable provider that builds nodes on demand.
struct Remote;

impl NodeProvider for Remote {
    fn get_node(&self, id: &NodeId) -> Option<Cow<'_, Node>> {
        (id == "r").then(|| Cow::Owned(node("r", "x")))
    }
}

#[test]
fn membership_is_edge_endpoints_then_enumerable_ids() {
    let graph = doc();
    assert_eq!(
        graph_node_ids(graph_source(&graph)),
        ["r", "p", "a", "lone"]
    );
}

#[test]
fn unresolved_ids_are_members_without_a_node() {
    let graph = doc();
    assert_eq!(unresolved_node_ids((&graph).into()), ["r", "p"]);
}

#[test]
fn layered_nodes_resolve_in_order_and_enumerate_only_enumerable_providers() {
    let graph = doc();
    let overrides: BTreeMap<NodeId, Node> = [("a".into(), node("a", "override"))].into();
    let remote = Remote;
    let nodes = LayeredNodes {
        providers: vec![&overrides, &graph.nodes, &remote],
    };
    assert_eq!(nodes.get_node(&"a".into()).unwrap().node_type, "override");
    assert_eq!(nodes.get_node(&"r".into()).unwrap().node_type, "x");
    assert_eq!(nodes.node_ids().unwrap(), ["a", "lone"]);
    let source = GraphSource {
        nodes: &nodes,
        edges: &graph.edges,
    };
    let ids: Vec<_> = resolved_nodes(source)
        .iter()
        .map(|n| n.id.clone())
        .collect();
    assert_eq!(ids, ["r", "a", "lone"]);
    assert!(LayeredNodes {
        providers: vec![&remote]
    }
    .node_ids()
    .is_none());
}

#[test]
fn inbound_edges_sort_by_order() {
    let graph = doc();
    let ids: Vec<_> = inbound_edges((&graph).into(), &"p".into(), &"items".into())
        .into_iter()
        .map(|(id, _)| id.clone())
        .collect();
    assert_eq!(ids, ["e1", "e2"]);
}

#[test]
fn materialize_keeps_every_edge_and_only_resolved_nodes() {
    let graph = doc();
    let flat = materialize_graph((&graph).into());
    assert_eq!(flat.nodes.keys().collect::<Vec<_>>(), ["a", "lone"]);
    assert_eq!(flat.edges.len(), 2);
}
