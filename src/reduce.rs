use crate::graph::{CircuitGraph, ComponentIndex, NodeIndex};
use std::collections::HashMap;
use crate::units::{AngularFrequency, ImpedanceResult, combine_parallel_many, combine_series};
use crate::component::impedance_to_kind;
use crate::errors::CircuitError;


#[derive(Clone)]
pub enum ReductionStep {
    Series{
        components: Vec<ComponentIndex>,
        equivalent: ComponentIndex,
        impedance: ImpedanceResult,
        nodes: (NodeIndex, NodeIndex),
    },
    Parallel{
        components: Vec<ComponentIndex>,
        equivalent: ComponentIndex,
        impedance: ImpedanceResult,
        nodes: (NodeIndex, NodeIndex),
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
            apply_reduction(graph, &mut step)?;
            steps.push(step);
            continue;
        }
        if let Some(mut step) = find_parallel_reduction(graph) {
            apply_reduction(graph, &mut step)?;
            steps.push(step);
            continue;
        }
        break;
    }
    Ok(steps)
}

fn find_series_reduction(graph: &CircuitGraph) -> Option<ReductionStep> {
    for node_idx in 0..graph.nodes.len() {
        
        if graph.get_node_degree(node_idx) != 2 {
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
            
        let middle_idx = if comp1.nodes.0 == comp2.nodes.0 || comp1.nodes.0 == comp2.nodes.1 {
            comp1.nodes.0
        } else {
            comp1.nodes.1
        };
            
        let outer1 = if comp1.nodes.0 != middle_idx { comp1.nodes.0 } else { comp1.nodes.1 };
        let outer2 = if comp2.nodes.0 != middle_idx { comp2.nodes.0 } else { comp2.nodes.1 };
        
        // Use combine_series helper
        let z_eq = combine_series(
            comp1.cached_impedance.clone()?,
            comp2.cached_impedance.clone()?,
        );
        
        // Return reduction even if result is Open/Short
        return Some(ReductionStep::Series {
            components: vec![comp1_idx, comp2_idx],
            equivalent: 0,
            impedance: z_eq,
            nodes: (outer1, outer2),
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
    let (nodes, best_indices) = parallel_groups
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .max_by_key(|(_, indices)| indices.len())?;
    
    let impedances: Vec<ImpedanceResult> = best_indices
        .iter()
        .map(|&idx| graph.components[idx].cached_impedance.clone())
        .collect::<Option<Vec<_>>>()?;
    
    let z_eq = combine_parallel_many(&impedances);
    
    return Some(ReductionStep::Parallel {
        components: best_indices,
        equivalent: 0,
        impedance: z_eq,
        nodes,
    });
}

fn apply_reduction(graph: &mut CircuitGraph, step: &mut ReductionStep) -> Result<ComponentIndex, CircuitError> {
    match step {
        ReductionStep::Series { components, impedance, equivalent, nodes} | 
        ReductionStep::Parallel { components, impedance, equivalent, nodes}
            => {
            for comp_idx in components {
                graph.components[*comp_idx].is_active = false;
            }
            
            let kind = impedance_to_kind(impedance.clone())?;
            let new_comp_idx = graph.add_component("EQ".to_string(), kind, *nodes);
            
            graph.components[new_comp_idx].cached_impedance = Some(impedance.clone());
            *equivalent = new_comp_idx;
            Ok(new_comp_idx)
        }
        _ => panic!("apply_reduction called with unimplemented reduction type")
    }
}
