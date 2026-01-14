use num_complex::Complex64;
use crate::component::ComponentKind;
use crate::units::AngularFrequency;

pub type NodeIndex = usize;
pub type ComponentIndex = usize;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub degree: usize,
}

#[derive(Debug, Clone)]
pub struct CircuitComponent {
    pub id: String,
    pub kind: ComponentKind,
    pub nodes: (NodeIndex, NodeIndex),
    pub is_active: bool,
    pub cached_impedance: Complex64,
}

#[derive(Debug, Clone)]
pub struct CircuitGraph {
    pub nodes: Vec<Node>,
    pub components: Vec<CircuitComponent>,
    pub ground: Option<NodeIndex>,
}

impl CircuitGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            components: Vec::new(),
            ground: None,
        }
    }

    pub fn add_node(&mut self, id: String) -> NodeIndex {
        let idx = self.nodes.len();
        self.nodes.push(Node { id, degree: 0 });
        idx
    }
    
    pub fn node(&self, idx: NodeIndex) -> &Node {
        &self.nodes[idx]
    }

    pub fn add_component(&mut self, id: String, kind: ComponentKind, nodes: (NodeIndex, NodeIndex)) -> ComponentIndex {
        debug_assert!(nodes.0 < self.nodes.len() && nodes.1 < self.nodes.len());
        self.nodes[nodes.0].degree += 1;
        self.nodes[nodes.1].degree += 1;
        let idx = self.components.len();
        self.components.push(CircuitComponent {
            id,
            kind,
            nodes,
            is_active: true,
            cached_impedance: Complex64::new(0.0, 0.0),
        });
        idx
    }

    pub fn component(&self, idx: ComponentIndex) -> &CircuitComponent {
        &self.components[idx]
    }

    pub fn connections_at(&self, idx: NodeIndex) -> Vec<ComponentIndex> {
        self.components
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_active && (c.nodes.0 == idx || c.nodes.1 == idx))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn active_component_count(&self) -> usize {
        self.components.iter().filter(|c| c.is_active).count()
    }

    pub fn cache_impedances(&mut self, omega: AngularFrequency) {
        for c in self.components.iter_mut() {
            if c.is_active {
                c.cached_impedance = c.kind.impedance(omega);
            }
        }
    }

    pub fn set_ground(&mut self, idx: NodeIndex) {
        self.ground = Some(idx);
    }

    pub fn is_ground(&self, idx: NodeIndex) -> bool {
        self.ground.map_or(false, |g| g == idx)
    }
}
