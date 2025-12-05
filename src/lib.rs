mod app;
mod config;
mod graphics;
mod model;
mod state;

pub use config::AppConfig;
pub use model::{Edge, ModeGraph, Network, Node, NodeType, TransportMode};

use app::App;
use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

/// Main application entry point. Platform-agnostic.
///
/// This function contains no platform-specific code. All platform
/// differences are handled by the calling shell (main.rs or wasm_start).
pub fn run_app(config: AppConfig) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let app = App::new_with_config(config);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        let _ = event_loop.run_app(&mut app);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = event_loop.spawn_app(app);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn wasm_start(val: wasm_bindgen::JsValue) {
    console_error_panic_hook::set_once();

    let config: AppConfig = if val.is_undefined() || val.is_null() {
        AppConfig::default()
    } else {
        serde_wasm_bindgen::from_value(val).unwrap_or_default()
    };

    run_app(config);
}
