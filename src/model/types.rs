use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportMode {
    Car,
    Bike,
    Walk,
    Transit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeType {
    Intersection,
    MidblockCrossing,
    TransitStop,
    Terminus,
}
