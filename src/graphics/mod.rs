pub mod rendering;
pub mod geometry;
pub mod scene;
pub mod ui;
pub mod shaders;

// Re-export public graphics API for state.rs to use
pub use rendering::{
    render_scene, CameraBuffer, GpuContext, InstanceBuffer,
    LightingBuffer, LightingControls, LightingSettings,
    MeshBuffers, Pipeline,
};
pub use ui::{panels, CameraDebugInfo, EguiIntegration};
