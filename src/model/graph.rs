use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::types::{NodeType, TransportMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub position: [f32; 2],
    pub node_type: NodeType,
    pub physical_attributes: Vec<String>,
    pub turn_restrictions: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: usize,
    pub from_node: usize,
    pub to_node: usize,
    pub facility_type: String,
    pub physical_attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeGraph {
    pub mode: TransportMode,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub graphs: HashMap<TransportMode, ModeGraph>,
}

impl Edge {
    /// Calculate edge length from node positions
    pub fn length(&self, graph: &ModeGraph) -> f32 {
        let from = &graph.nodes[self.from_node].position;
        let to = &graph.nodes[self.to_node].position;
        let dx = to[0] - from[0];
        let dz = to[1] - from[1];
        (dx * dx + dz * dz).sqrt()
    }
}

impl Network {
    pub fn new() -> Self {
        Self {
            graphs: HashMap::new(),
        }
    }
}
