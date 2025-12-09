use super::{Scene, SceneNode};
use glam::Vec3;

#[derive(Debug, Default)]
pub struct PickingState {
    /// Currently hovered node ID (continuously updated)
    pub hovered_node: Option<u32>,
    /// Currently picked node ID (locked during drag)
    pub picked_node: Option<u32>,
    drag: Option<DragState>,
}

#[derive(Debug, Clone, Copy)]
struct DragState {
    last_mouse_pos: (f32, f32),
    node_locked: bool,
    drag_offset: Option<Vec3>,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            last_mouse_pos: (0.0, 0.0),
            node_locked: false,
            drag_offset: None,
        }
    }
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
            node_locked: false,
            drag_offset: None,
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

    /// Lock the current drag to a specific node with an offset
    pub fn lock_node_with_offset(&mut self, offset: Vec3) {
        if let Some(drag) = self.drag.as_mut() {
            drag.node_locked = true;
            drag.drag_offset = Some(offset);
        }
    }

    /// Check if the currently dragged node is locked
    pub fn is_node_locked(&self) -> bool {
        self.drag.as_ref().map_or(false, |d| d.node_locked)
    }

    /// Get the drag offset if available
    pub fn get_drag_offset(&self) -> Option<Vec3> {
        self.drag.as_ref().and_then(|d| d.drag_offset)
    }

    /// Update the hovered node (continuously updated, independent of drag)
    pub fn update_hovered_node(&mut self, node_id: Option<u32>) {
        self.hovered_node = node_id;
    }
}
