use super::{Scene, SceneNode};
#[derive(Debug, Default)]
pub struct PickingState {
    /// Currently picked node ID, or None if no node is picked
    pub picked_node: Option<u32>,
    drag: Option<DragState>,
}

#[derive(Debug, Default, Clone, Copy)]
struct DragState {
    last_mouse_pos: (f32, f32),
}

impl PickingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the picking state with a new picked node
    pub fn update_picked_node(&mut self, node_id: Option<u32>) -> bool {
        let changed = self.picked_node != node_id;
        self.picked_node = node_id;
        changed
    }

    /// Get a reference to the currently picked node, if any
    pub fn get_picked_node<'a>(&self, scene: &'a Scene) -> Option<&'a SceneNode> {
        self.picked_node.and_then(|id| {
            if (id as usize) < scene.nodes.len() {
                Some(&scene.nodes[id as usize])
            } else {
                None
            }
        })
    }

    /// Begin tracking a drag at the given mouse position
    pub fn start_drag(&mut self, mouse_pos: (f32, f32)) {
        self.drag = Some(DragState {
            last_mouse_pos: mouse_pos,
        });
    }

    /// Update drag with a new mouse position, returning the delta since the last update
    pub fn update_drag(&mut self, mouse_pos: (f32, f32)) -> Option<(f32, f32)> {
        let drag = self.drag.as_mut()?;
        let delta = (
            mouse_pos.0 - drag.last_mouse_pos.0,
            mouse_pos.1 - drag.last_mouse_pos.1,
        );
        drag.last_mouse_pos = mouse_pos;
        Some(delta)
    }

    /// End a drag operation
    pub fn end_drag(&mut self) {
        self.drag = None;
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }
}
