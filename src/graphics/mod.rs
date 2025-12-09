pub mod geometry;
pub mod rendering;
pub mod scene;
pub mod shaders;
pub mod ui;

// Re-export public graphics API for state.rs to use
pub use rendering::{
    CameraBuffer, GpuContext, InstanceBuffer, InstanceData, LightingBuffer, LightingControls,
    LightingSettings, MeshBuffers, Pipeline, render_scene,
};
pub use ui::{CameraDebugInfo, EguiIntegration, RenderStats, panels};
