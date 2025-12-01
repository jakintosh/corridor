mod app;
mod geometry;
mod rendering;
mod shaders;
mod state;
mod ui;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
