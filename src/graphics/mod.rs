pub mod geometry;
pub mod rendering;
pub mod scene;
pub mod shaders;
pub mod ui;

// Re-export public graphics API for state.rs to use
pub use rendering::{
    render_scene, CameraBuffer, GpuContext, InstanceBuffer, LightingBuffer, LightingControls,
    LightingSettings, MeshBuffers, PickingPass, Pipeline,
};
pub use ui::{panels, CameraDebugInfo, EguiIntegration};
