use proptest::prelude::*;
use num_complex::Complex64;

use electro_solve_core::units::*;
use electro_solve_core::graph::*;
use electro_solve_core::component::*;
use electro_solve_core::reduce::*;

mod common;
use common::*;
use common::strategies::*;

proptest! {

#[test]
fn prop_series_chain_reduces_to_sum(
    values in prop::collection::vec(1.0_f64..1e6_f64, 2..20)
) {
    let mut graph = CircuitGraph::new();
    let omega = AngularFrequency::new(100.0).unwrap();

    for i in 0..=values.len() {
        graph.add_node(format!("n{}", i));
    }

    for (i, &val) in values.iter().enumerate() {
        let kind = ComponentKind::Resistor { 
            r: Resistance::known(val).unwrap() 
        };
        graph.add_component(format!("R{}", i), kind, (i, i + 1));
    }
    let expected_r: f64 = values.iter().sum();
    let expected_z = Complex64::new(expected_r, 0.0);

    let _ = reduce(&mut graph, omega).expect("Should reduce successfully");

    prop_assert_eq!(graph.active_component_count(), 1, "Should have one active component");

    let remaining_comp = graph.components
        .iter()
        .find(|c| c.is_active)
        .expect("Active count was 1, but could not find active component");
    
    let z_result = remaining_comp.cached_impedance.as_ref().unwrap();
    
    match z_result {
        ImpedanceResult::Finite(z) => {
            // Check Real part matches sum
            prop_assert!( (z.re - expected_z.re).abs() < 1e-6, 
                "Expected {} Ohms, got {} Ohms", expected_z.re, z.re);
            // Check Imaginary part is 0
            prop_assert!( z.im.abs() < 1e-6 );
        },
        _ => prop_assert!(false, "Result should be finite")
    }
}


#[test]
fn prop_parallel_bank_reduces_to_harmonic_mean(
    values in prop::collection::vec(1.0_f64..1e6_f64, 2..20)
) {
    let mut graph = CircuitGraph::new();
    let omega = AngularFrequency::new(100.0).unwrap();

    graph.add_node("n0".to_string());
    graph.add_node("n1".to_string());
    graph.set_ground(0);


    for (i, &val) in values.iter().enumerate() {
        let kind = ComponentKind::Resistor { 
            r: Resistance::known(val).unwrap() 
        };
        graph.add_component(format!("R{}", i), kind, (0, 1));
    }

    let sum_conductance: f64 = values.iter().map(|&r| 1.0 / r).sum();
    let expected_r: f64 = 1.0 / sum_conductance;
    let expected_z = Complex64::new(expected_r, 0.0);

    let _ = reduce(&mut graph, omega).expect("Should reduce successfully");

    prop_assert_eq!(graph.active_component_count(), 1, "Should have one active component");

    let remaining_comp = graph.components
        .iter()
        .find(|c| c.is_active)
        .expect("Active count was 1, but could not find active component");
    
    let z_result = remaining_comp.cached_impedance.as_ref().unwrap();
    
    match z_result {
        ImpedanceResult::Finite(z) => {
            // Check Real part matches sum
            prop_assert!( (z.re - expected_z.re).abs() < 1e-6, 
                "Expected {} Ohms, got {} Ohms", expected_z.re, z.re);
            // Check Imaginary part is 0
            prop_assert!( z.im.abs() < 1e-6 );
        },
        _ => prop_assert!(false, "Result should be finite")
    }
}

#[test]
fn prop_reduction_never_creates_self_loops(
    mut graph in arbitrary_circuit_graph()
) {
    let omega = AngularFrequency::new(100.0).unwrap();
    let _ = reduce(&mut graph, omega);

    for comp in &graph.components {
        if comp.is_active {
            prop_assert_ne!(comp.nodes.0, comp.nodes.1, 
                "Reduction created a self-loop on component {}", comp.id);
        }
    }
}

}
