mod app;
mod geometry;
mod rendering;
mod scene;
mod shaders;
mod state;
mod ui;

use app::App;
use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let app = App::new();
    let _ = event_loop.spawn_app(app);
}
