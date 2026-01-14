use num_complex::Complex64;
use crate::graph::{CircuitGraph, ComponentIndex, NodeIndex};
use std::collections::HashMap;
use crate::units::AngularFrequency;
use crate::component::impedance_to_kind;
use crate::errors::CircuitError;

pub enum ReductionStep {
    Series{
        components: Vec<ComponentIndex>,
        equivalent: ComponentIndex,
        impedance: Complex64,
    },
    Parallel{
        components: Vec<ComponentIndex>,
        equivalent: ComponentIndex,
        impedance: Complex64,
    },
    DeltaWye {
        delta_nodes: (NodeIndex, NodeIndex, NodeIndex),
        wye_node: NodeIndex,
    }
}


pub fn reduce(graph: &mut CircuitGraph, omega: AngularFrequency) -> Result<Vec<ReductionStep>, CircuitError> {
    let mut steps = Vec::new();
    graph.cache_impedances(omega);
    loop {
        if let Some(mut step) = find_series_reduction(graph) {
            apply_reduction(graph, &mut step);
            steps.push(step);
            continue;
        }
        if let Some(mut step) = find_parallel_reduction(graph) {
            apply_reduction(graph, &mut step);
            steps.push(step);
            continue;
        }
        break;
    }
    Ok(steps)
}

fn find_series_reduction(graph: &CircuitGraph) -> Option<ReductionStep> {
    for node_idx in 0..graph.nodes.len() {
        let node = &graph.nodes[node_idx];
        
        if node.degree != 2 {
            continue;
        }
        
        let connected = graph.connections_at(node_idx);
        
        if connected.len() != 2 {
            continue;
        }
        
        let comp1_idx = connected[0];
        let comp2_idx = connected[1];
        
        let comp1 = &graph.components[comp1_idx];
        let comp2 = &graph.components[comp2_idx];
        
        if !comp1.kind.is_passive() || !comp2.kind.is_passive() {
            continue;
        }
        
        if comp1.cached_impedance == Complex64::new(0.0, 0.0) ||
           comp2.cached_impedance == Complex64::new(0.0, 0.0) {
            continue;
        }
        
        let z_eq = comp1.cached_impedance + comp2.cached_impedance;
        
        return Some(ReductionStep::Series {
            components: vec![comp1_idx, comp2_idx],
            equivalent: 0,
            impedance: z_eq,
        });
    }
    None
}

fn find_parallel_reduction(graph: &CircuitGraph) -> Option<ReductionStep> {
    let mut parallel_groups: HashMap<(NodeIndex, NodeIndex), Vec<ComponentIndex>> = HashMap::new();
    for (idx, comp) in graph.components.iter().enumerate() {
        if !comp.is_active || !comp.kind.is_passive() {
            continue;
        }
    
        let key = if comp.nodes.0 < comp.nodes.1 {
            comp.nodes
        } else {
            (comp.nodes.1, comp.nodes.0)
        };
    
        parallel_groups.entry(key).or_default().push(idx);
    }

    let (_, best_indices) = parallel_groups
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .max_by_key(|(_, indices)| indices.len())?;
    
    // Then calculate impedance using best_indices
    let mut admittance_sum = Complex64::new(0.0, 0.0);
    for &idx in &best_indices {
        let z = graph.components[idx].cached_impedance;
        if z == Complex64::new(0.0, 0.0) {
            continue;
        }
        admittance_sum += 1.0 / z;
    }
    let z_eq = 1.0 / admittance_sum;
    return Some(ReductionStep::Parallel {
        components: best_indices,
        equivalent: 0,
        impedance: z_eq,
    });
}

fn apply_reduction(graph: &mut CircuitGraph, step: &mut ReductionStep) -> Result<ComponentIndex, CircuitError> {
    match step {
        ReductionStep::Series { components, impedance, equivalent } => {
            let comp1 = &graph.components[components[0]];
            let comp2 = &graph.components[components[1]];
            
            let middle_idx = if comp1.nodes.0 == comp2.nodes.0 || comp1.nodes.0 == comp2.nodes.1 {
                comp1.nodes.0
            } else {
                comp1.nodes.1
            };
            let outer1 = if comp1.nodes.0 != middle_idx { comp1.nodes.0 } else { comp1.nodes.1 };
            let outer2 = if comp2.nodes.0 != middle_idx { comp2.nodes.0 } else { comp2.nodes.1 };
            
            for comp_idx in components {
                graph.components[*comp_idx].is_active = false;
                let nodes = graph.components[*comp_idx].nodes;
                graph.nodes[nodes.0].degree -= 1;
                graph.nodes[nodes.1].degree -= 1;
            }
            
            let kind = impedance_to_kind(*impedance)?;
            let new_comp_idx = graph.add_component("EQ".to_string(), kind, (outer1, outer2));
            
            *equivalent = new_comp_idx;
            Ok(new_comp_idx)
        }
        
        ReductionStep::Parallel { components, impedance, equivalent } => {
            let comp = &graph.components[components[0]];
            let nodes = comp.nodes;
            
            for comp_idx in components {
                graph.components[*comp_idx].is_active = false;
                let c_nodes = graph.components[*comp_idx].nodes;
                graph.nodes[c_nodes.0].degree -= 1;
                graph.nodes[c_nodes.1].degree -= 1;
            }
            
            let kind = impedance_to_kind(*impedance)?;
            let new_comp_idx = graph.add_component("EQ".to_string(), kind, nodes);
            
            *equivalent = new_comp_idx;
            Ok(new_comp_idx)
        }
        
        _ => panic!("apply_reduction called with unimplemented reduction type")
    }
}
