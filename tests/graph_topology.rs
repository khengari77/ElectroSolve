use proptest::prelude::*;
use electro_solve::units::*;
use electro_solve::graph::*;
use electro_solve::component::*;

fn arbitrary_component_kind() -> impl Strategy<Value = ComponentKind> {
    prop_oneof![
        (1e-12_f64..1e12_f64).prop_map(|r| ComponentKind::Resistor{r: Resistance::known(r).unwrap()}),
        (1e-12_f64..1e12_f64).prop_map(|l| ComponentKind::Inductor{l: Inductance::known(l).unwrap()}),
        (1e-12_f64..1e12_f64).prop_map(|c| ComponentKind::Capacitor{c: Capacitance::known(c).unwrap()}),
        (1e-12_f64..1e12_f64).prop_map(|v| ComponentKind::VoltageSource{v: Voltage::dc(v)}),
        (1e-12_f64..1e12_f64).prop_map(|i| ComponentKind::CurrentSource{i: Current::dc(i)}),
    ]
}

fn arbitrary_circuit_graph() -> impl Strategy<Value = CircuitGraph> {
    prop::collection::vec("[a-z]{1,5}", 2..=10)
        .prop_flat_map(|node_ids| {
            let num_components = 0usize..=15usize;
            prop::collection::vec(
                (arbitrary_component_kind(), 
                 (0usize..node_ids.len(), 0usize..node_ids.len())
                    .prop_filter("distinct nodes", |(n0, n1)| n0 != n1)),
                num_components
            )
            .prop_map(move |components| (node_ids.clone(), components))
        })
        .prop_map(|(node_ids, components)| {
            let mut graph = CircuitGraph::new();
            for id in &node_ids {
                graph.add_node(id.clone());
            }
            // Initialize adjacency for all nodes if components is empty
            if components.is_empty() {
                for _ in 0..graph.nodes.len() {
                    graph.adjacency.push(Vec::new());
                }
            }
            for (i, (kind, (n0, n1))) in components.into_iter().enumerate() {
                graph.add_component(format!("C{}", i), kind, (n0, n1));
            }
            graph
        })
}

proptest! {

#[test]
fn prop_adjacency_is_bidirectional(
    graph in arbitrary_circuit_graph()
) {
    for (comp_idx, comp) in graph.components.iter().enumerate() {
        if comp.is_active {
            let connections_0 = graph.connections_at(comp.nodes.0);
            let connections_1 = graph.connections_at(comp.nodes.1);
            prop_assert!(connections_0.contains(&comp_idx));
            prop_assert!(connections_1.contains(&comp_idx));
        }
    }
}
#[test]
fn prop_node_degree_matches_connections(
    input in arbitrary_circuit_graph()
        .prop_flat_map(|graph| {
            let node_count = graph.nodes.len();
            (Just(graph), 0usize..node_count)
        })
    ) {
    let (mut graph, node_idx) = input;
    let degree = graph.get_node_degree(node_idx);
    let connections = graph.connections_at(node_idx);
    prop_assert_eq!(degree, connections.len());
}
#[test]
fn prop_connections_only_return_active(
    input in arbitrary_circuit_graph()
        .prop_flat_map(|graph| {
            let node_count = graph.nodes.len();
            (Just(graph), 0usize..node_count)
        })
    ) {
    let (mut graph, node_idx) = input;
    for comp_idx in graph.connections_at(node_idx) {
        let component = graph.component(comp_idx);
        prop_assert!(component.is_active);
    }
}
#[test]
fn prop_add_node_sequential_indices(
    mut graph in arbitrary_circuit_graph(),
    node_id in "[a-z]{1,5}"
) {
    let before_count = graph.nodes.len();
    let idx = graph.add_node(node_id);
    prop_assert_eq!(idx, before_count);
}
#[test]
fn prop_add_component_sequential_indices(
    input in (arbitrary_circuit_graph(), arbitrary_component_kind())
        .prop_flat_map(|(graph, kind)| {
            let node_count = graph.nodes.len();
            let indices = prop::collection::vec(0usize..node_count, 2);
            (Just(graph), Just(kind), indices)
        })
) {
    let (mut graph, kind, node_indices) = input;

    if graph.nodes.len() >= 2 {
        let before_count = graph.components.len();
        let idx = graph.add_component(
            "test".to_string(),
            kind,
            (node_indices[0], node_indices[1])
        );
        prop_assert_eq!(idx, before_count);
    }
}
#[test]
fn prop_node_retrieval_returns_added(
    mut graph in arbitrary_circuit_graph(),
    node_id in "[a-z]{1,5}"
) {
    let idx = graph.add_node(node_id.clone());
    let node = graph.node(idx);
    prop_assert_eq!(node.id.clone(), node_id);
}
#[test]
fn prop_component_retrieval_consistent(
    graph in arbitrary_circuit_graph()
) {
    for (idx, component) in graph.components.iter().enumerate() {
        let retrieved = graph.component(idx);
        prop_assert_eq!(retrieved.kind.clone(), component.kind.clone());
    }
}
#[test]
fn prop_ground_is_unique(
    mut graph in arbitrary_circuit_graph()
) {
    if graph.nodes.len() > 0 {
        let gnd_idx = graph.nodes.len() / 2;
        graph.set_ground(gnd_idx);
        prop_assert!(graph.is_ground(gnd_idx));
        
        for idx in 0..graph.nodes.len() {
            if idx != gnd_idx {
                prop_assert!(!graph.is_ground(idx));
            }
        }
    }
}
#[test]
fn prop_active_component_count_matches_filter(
    graph in arbitrary_circuit_graph()
) {
    let count = graph.active_component_count();
    let filtered_count = graph.components.iter()
        .filter(|c| c.is_active)
        .count();
    prop_assert_eq!(count, filtered_count);
}
#[test]
fn prop_cache_impedances_updates_all_active(
    graph in arbitrary_circuit_graph(),
    omega in 1.0_f64..1e6_f64
) {
    let mut graph = graph;
    graph.cache_impedances(AngularFrequency::new(omega).unwrap());
    for component in &graph.components {
        if component.is_active {
            prop_assert!(component.cached_impedance.is_some());
        }
    }
}
#[test]
fn prop_cache_impedances_skips_inactive(
    graph in arbitrary_circuit_graph(),
    omega in 1.0_f64..1e6_f64
) {
    let mut graph = graph;
    graph.cache_impedances(AngularFrequency::new(omega).unwrap());
    for component in &graph.components {
        if !component.is_active {
            prop_assert!(component.cached_impedance.is_none());
        }
    }
}
#[test]
fn prop_deactivating_component_reduces_count(
    input in arbitrary_circuit_graph()
        .prop_filter("has components", |graph| !graph.components.is_empty())
        .prop_flat_map(|graph| {
            let comp_count = graph.components.len();
            (Just(graph), 0usize..comp_count)  // Generate graph first, then index
        })
) {
    let (mut graph, comp_idx) = input;
    let before_count = graph.active_component_count();
    graph.components[comp_idx].is_active = false;
    let after_count = graph.active_component_count();
    prop_assert_eq!(after_count, before_count.saturating_sub(1));
}
#[test]
fn prop_empty_graph_has_no_connections(
    graph in prop::strategy::Just(CircuitGraph::new())
) {
    prop_assert_eq!(graph.nodes.len(), 0);
    prop_assert_eq!(graph.components.len(), 0);
    for idx in 0..0 {
        prop_assert!(graph.connections_at(idx).is_empty());
    }
}
#[test]
fn prop_ground_persists_after_operations(
    mut graph in arbitrary_circuit_graph(),
    gnd_idx in 0usize..10usize
) {
    if graph.nodes.len() > gnd_idx {
        graph.set_ground(gnd_idx);
        
        let _ = graph.add_node("extra".to_string());
        
        prop_assert!(graph.is_ground(gnd_idx));
    }
}

}
