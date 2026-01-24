/// strategies.rs
use proptest::prelude::*;
use num_complex::Complex64;
use electro_solve_core::units::*;
use electro_solve_core::component::*;
use electro_solve_core::graph::*;
use electro_solve_core::reduce::*;


/// Generate arbitrary impedance values across physical range
pub fn impedance_strategy() -> impl Strategy<Value = ImpedanceResult> {
    prop_oneof![
        (1e-12_f64..1e12_f64, -1e12_f64..1e12_f64)
            .prop_map(|(re, im)| ImpedanceResult::new_finite(Complex64::new(re, im)))
            .boxed(),
    ]
}

/// Generate arbitrary component kinds with physical values
pub fn arbitrary_component_kind() -> impl Strategy<Value = ComponentKind> {
    prop_oneof![
        (1e-12_f64..1e12_f64).prop_map(|r| ComponentKind::Resistor{r: Resistance::known(r).unwrap()}),
        (1e-12_f64..1e12_f64).prop_map(|l| ComponentKind::Inductor{l: Inductance::known(l).unwrap()}),
        (1e-12_f64..1e12_f64).prop_map(|c| ComponentKind::Capacitor{c: Capacitance::known(c).unwrap()}),
        (1e-12_f64..1e12_f64).prop_map(|v| ComponentKind::VoltageSource{v: Voltage::dc(v)}),
        (1e-12_f64..1e12_f64).prop_map(|i| ComponentKind::CurrentSource{i: Current::dc(i)}),
    ]
}

/// Generate arbitrary circuit graphs with guaranteed connectivity
pub fn arbitrary_circuit_graph() -> impl Strategy<Value = CircuitGraph> {
    prop::collection::vec("[a-z]{1,5}", 2..=10)
        .prop_flat_map(|node_ids| {
            let num_components = 1usize..=15usize;
            prop::collection::vec(
                (arbitrary_component_kind(), 
                 (0usize..node_ids.len(), 0usize..node_ids.len())
                    .prop_filter("distinct nodes", |(n0, n1)| n0 != n1)),
                num_components
            )
            .prop_map(move |components| (node_ids.clone(), components))
        })
        .prop_map(|(node_ids, components): (Vec<String>, Vec<(ComponentKind, (usize, usize))>)| {
            let mut graph = CircuitGraph::new();
            for id in &node_ids {
                graph.add_node(id.clone());
            }
            for (i, (kind, (n0, n1))) in components.into_iter().enumerate() {
                graph.add_component(format!("C{}", i), kind, (n0, n1));
            }
            graph
        })
}

/// Generate arbitrary angular frequencies
pub fn frequency_strategy() -> impl Strategy<Value = AngularFrequency> {
    (1.0_f64..1e6_f64).prop_map(|omega| AngularFrequency::new(omega).unwrap())
}

pub fn arbitrary_value() -> impl Strategy<Value = Value<f64>> {
    prop_oneof![
    (1e-12_f64..1e12_f64).prop_map(|v| Value::Known(v))
        .boxed(),
    prop::collection::vec("[a-z]{1,5}", 2..=10)
        .prop_map(|v| Value::Unknown(v.join("")))
        .boxed(),
    ]   
}   
