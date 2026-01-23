use proptest::prelude::*;
use num_complex::Complex64;

use electro_solve::units::*;
use electro_solve::graph::*;
use electro_solve::component::*;
use electro_solve::reduce::*;

mod common;
use common::*;
use common::strategies::*;

proptest! {

#[test]
fn prop_series_reduction_preserves_impedance(
    values in prop::collection::vec(1.0_f64..1e6_f64, 2..10)
) {
    // Build series chain
    let mut graph = CircuitGraph::new();
    let _ = build_series_chain(&mut graph, &values);
    
    let omega = AngularFrequency::new(1.0).unwrap();
    
    // We expect the reduction to squash the chain.
    let expected_z = series_impedance(&values);
    
    // Use the main reduce function which handles finding, calculating impedance, and applying
    let steps = reduce(&mut graph, omega).expect("Should reduce successfully");
    
    // Verify that we actually performed reductions
    assert!(!steps.is_empty(), "Should perform reduction on series chain");
    
    // Verify that at least one step was a Series reduction (topology check)
    assert!(steps.iter().any(|step| matches!(step, ReductionStep::Series{..})), 
            "Should contain a Series reduction step");

    // Verify final equivalent impedance
    let final_z = calculate_equivalent_impedance(&graph, omega);
    assert_impedance_eq(final_z, expected_z, EPSILON_PHYSICAL);
}

#[test]
fn prop_parallel_reduction_preserves_impedance(
    values in prop::collection::vec(1.0_f64..1e6_f64, 2..5)
) {
    // Build parallel bank
    let mut graph = CircuitGraph::new();
    graph.add_node("n0".to_string());
    graph.add_node("n1".to_string());
    
    for (i, &val) in values.iter().enumerate() {
        create_resistor(&mut graph, &format!("R{}", i), val, 0, 1);
    }
    
    let omega = AngularFrequency::new(1.0).unwrap();
    
    // Calculate expected impedance
    let impedances: Vec<_> = values.iter()
        .map(|&r| ImpedanceResult::new_finite(Complex64::new(r, 0.0)))
        .collect();
    let expected_z = combine_parallel_many(&impedances);
    
    // Use the main reduce function
    let steps = reduce(&mut graph, omega).expect("Should reduce successfully");
    
    // Verify reduction occurred
    assert!(!steps.is_empty(), "Should perform reduction on parallel bank");

    // Verify topology (at least one Parallel step)
    assert!(steps.iter().any(|step| matches!(step, ReductionStep::Parallel{..})), 
            "Should contain a Parallel reduction step");
    
    // Verify impedance
    let final_z = calculate_equivalent_impedance(&graph, omega);
    assert_impedance_eq(final_z, expected_z, EPSILON_PHYSICAL);
}

#[test]
fn prop_full_reduction_preserves_equivalent_impedance(
    circuit in arbitrary_circuit_graph_with_reducible_pairs()
) {
    // We use the generated circuit which is guaranteed to be reducible.
    // No prop_assume! needed.
    
    // Calculate initial equivalent impedance
    let omega = AngularFrequency::new(1.0).unwrap();
    let initial_z = calculate_equivalent_impedance(&circuit, omega);
    
    // Reduce circuit
    let mut graph = circuit;
    let steps = reduce(&mut graph, omega).expect("Should reduce successfully");
    
    // Verify at least one reduction occurred
    prop_assert!(!steps.is_empty(), "Circuit generated with reducible pairs should reduce");
    
    // Calculate final equivalent impedance
    let final_z = calculate_equivalent_impedance(&graph, omega);
    
    // Verify impedance preserved
    assert_impedance_eq(final_z, initial_z, EPSILON_RELAXED);
}

#[test]
fn prop_reduction_produces_valid_topology(
    circuit in arbitrary_circuit_graph_with_reducible_pairs()
) {
    // Reduce circuit
    let mut graph = circuit;
    let omega = AngularFrequency::new(1.0).unwrap();
    let _ = reduce(&mut graph, omega).expect("Should reduce successfully");
    
    // Verify topology validity
    for (idx, comp) in graph.components.iter().enumerate() {
        if comp.is_active {
            // Verify node indices are valid
            prop_assert!(comp.nodes.0 < graph.nodes.len());
            prop_assert!(comp.nodes.1 < graph.nodes.len());
            
            // Verify nodes are distinct
            prop_assert!(comp.nodes.0 != comp.nodes.1,
                        "Component {} has same node twice", idx);
        }
    }
    
    // Verify adjacency is fully initialized/valid
    prop_assert_eq!(graph.len_adjacency(), graph.nodes.len(),
                   "Adjacency array should match node count");
}

#[test]
fn prop_reduction_preserves_passivity(
    circuit in arbitrary_circuit_graph_with_reducible_pairs()
) {
    // Use the robust strategy
    
    let omega = AngularFrequency::new(1.0).unwrap();
    
    // Verify initial passivity (our generator uses passive components mostly)
    let initial_z = calculate_equivalent_impedance(&circuit, omega);
    // Only proceed if random graph turned out passive (it usually is unless sources are weird)
    prop_assume!(is_passive_impedance_result(&initial_z),
                "Initial circuit must be passive");
    
    // Reduce circuit
    let mut graph = circuit;
    let _ = reduce(&mut graph, omega).expect("Should reduce successfully");
    
    // Verify final passivity
    let final_z = calculate_equivalent_impedance(&graph, omega);
    prop_assert!(is_passive_impedance_result(&final_z),
                "Reduced circuit should remain passive");
}

#[test]
fn prop_reduction_is_idempotent(
    circuit in arbitrary_circuit_graph_with_reducible_pairs()
) {
    let omega = AngularFrequency::new(1.0).unwrap();
    
    // First reduction
    let mut graph1 = circuit.clone();
    let steps1 = reduce(&mut graph1, omega).expect("Should reduce successfully");
    
    // Second reduction (should produce same result or further reduce if first pass wasn't exhaustive, 
    // but reduce() is usually exhaustive. If reduce() is exhaustive, graph1 should be stable.)
    // However, the test name "idempotent" often implies f(f(x)) = f(x).
    // Let's check if running reduce again on graph1 produces no steps.
    
    let steps_again = reduce(&mut graph1, omega).expect("Should reduce again");
    prop_assert!(steps_again.is_empty(), "Full reduction should be exhaustive (idempotent)");
    
    // Alternatively, comparing two parallel runs:
    // This part of the original test checked if graph1 (reduced) is equivalent to graph2 (reduced).
    // Which is trivially true if deterministic. 
    // The previous implementation compared steps length and impedance.
    
    let mut graph2 = circuit;
    let steps2 = reduce(&mut graph2, omega).expect("Should reduce successfully");
    
    prop_assert_eq!(steps1.len(), steps2.len());
    
    let z1 = calculate_equivalent_impedance(&graph1, omega);
    let z2 = calculate_equivalent_impedance(&graph2, omega);
    assert_impedance_eq(z1, z2, EPSILON_STRICT);
}

}
