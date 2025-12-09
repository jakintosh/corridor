use crate::graphics::rendering::LightingControls;
use egui::Ui;

#[derive(Clone, Copy)]
pub struct CameraDebugInfo {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub object_count: usize,
}

#[derive(Clone, Copy)]
pub struct RenderStats {
    pub node_count: usize,
    pub vertex_count: usize,
    pub material_count: usize,
    pub current_fps: f32,
    pub avg_fps_1s: f32,
    pub avg_fps_5s: f32,
}

pub fn camera_debug(ui: &mut Ui, info: &CameraDebugInfo) {
    ui.label("Camera");
    ui.label("Left-click and drag to orbit camera");
    ui.label("Scroll wheel to zoom in/out");
    ui.separator();
    ui.monospace(format!(
        "pos({:.2}, {:.2}, {:.2})",
        info.position[0], info.position[1], info.position[2]
    ));
    ui.monospace(format!(
        "look({:.2}, {:.2}, {:.2})",
        info.target[0], info.target[1], info.target[2]
    ));
    ui.monospace(format!(
        "yaw:{:.2} pitch:{:.2} dist:{:.2}",
        info.yaw, info.pitch, info.distance
    ));
    ui.monospace(format!("objects: {}", info.object_count));
}

pub fn lighting(ui: &mut Ui, controls: &mut LightingControls) {
    ui.label("Lighting");

    // Sun color
    let mut sun_color = controls.sun_color.to_array();
    if ui.color_edit_button_rgb(&mut sun_color).changed() {
        controls.sun_color = glam::Vec3::from_array(sun_color);
    }

    // Sun intensity
    ui.add(egui::Slider::new(&mut controls.sun_intensity, 0.0..=5.0).text("Sun intensity"));

    // Sun direction
    let mut dir = controls.sun_direction.to_array();
    ui.horizontal(|ui| {
        ui.label("Sun dir");
        ui.add(
            egui::DragValue::new(&mut dir[0])
                .speed(0.01)
                .range(-1.0..=1.0),
        );
        ui.add(
            egui::DragValue::new(&mut dir[1])
                .speed(0.01)
                .range(-1.0..=1.0),
        );
        ui.add(
            egui::DragValue::new(&mut dir[2])
                .speed(0.01)
                .range(-1.0..=1.0),
        );
    });
    controls.sun_direction = glam::Vec3::from_array(dir);

    // Horizon color
    let mut horizon_color = controls.horizon_color.to_array();
    if ui.color_edit_button_rgb(&mut horizon_color).changed() {
        controls.horizon_color = glam::Vec3::from_array(horizon_color);
    }

    ui.add(egui::Slider::new(&mut controls.ambient_height, 0.1..=20.0).text("Ambient height"));
}

pub fn hover_info(ui: &mut Ui, hovered_node_id: Option<u32>) {
    ui.label("Hover");
    match hovered_node_id {
        Some(node_id) => {
            ui.monospace(format!("Hovered Node ID: {}", node_id));
        }
        None => {
            ui.monospace("Hovered Node ID: None");
        }
    }
}

pub fn render_stats(ui: &mut Ui, stats: &RenderStats) {
    ui.label("Rendering Statistics");
    ui.monospace(format!(
        "FPS: {:.1} (1s: {:.1}, 5s: {:.1})",
        stats.current_fps, stats.avg_fps_1s, stats.avg_fps_5s
    ));
    ui.monospace(format!("Nodes: {}", stats.node_count));
    ui.monospace(format!("Vertices: {}", stats.vertex_count));
    ui.monospace(format!("Materials: {}", stats.material_count));
}
