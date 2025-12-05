use crate::model::Network;
use serde::{Deserialize, Serialize};

/// Application configuration that can be passed from any entry point.
///
/// This struct is designed to be:
/// - Serializable for persistence and WASM interop
/// - Extensible for future configuration options
/// - Platform-agnostic (no target-specific fields)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Optional network to visualize. If None, a demo scene is shown.
    #[serde(default)]
    pub network: Option<Network>,
}

impl AppConfig {
    /// Create a config with the given network
    pub fn with_network(network: Network) -> Self {
        Self {
            network: Some(network),
        }
    }
}
