use crate::config::AppConfig;
use crate::state::State;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App {
    state: Option<State>,
    window: Option<std::sync::Arc<Window>>,
    config: AppConfig,
}

impl App {
    pub fn new() -> Self {
        Self::new_with_config(AppConfig::default())
    }

    pub fn new_with_config(config: AppConfig) -> Self {
        Self {
            state: None,
            window: None,
            config,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn setup_canvas_size(window: &Window) {
        use wasm_bindgen::JsCast;
        use winit::dpi::LogicalSize;
        use winit::platform::web::WindowExtWebSys;

        let (width, height) = {
            let win = web_sys::window();
            let w = win
                .as_ref()
                .and_then(|w| w.inner_width().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(1024.0);
            let h = win
                .as_ref()
                .and_then(|w| w.inner_height().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(768.0);
            (w, h)
        };

        // Update winit window logical size (which drives surface config).
        let _ = window.request_inner_size(LogicalSize::new(width, height));

        // Keep the canvas actual pixel size in sync for correct presentation.
        if let Some(canvas) = window.canvas() {
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);

            // Stretch to fill the page.
            if let Some(elem) = canvas.dyn_ref::<web_sys::HtmlElement>() {
                let style = elem.style();
                let _ = style.set_property("width", "100vw");
                let _ = style.set_property("height", "100vh");
                let _ = style.set_property("display", "block");
                let _ = style.set_property("margin", "0");
                let _ = style.set_property("padding", "0");
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn attach_canvas(&self) {
        use winit::platform::web::WindowExtWebSys;

        if let Some(window) = self.window.as_ref() {
            if let Some(canvas) = window.canvas() {
                let window = web_sys::window().expect("no global `window` exists");
                let document = window.document().expect("should have a document on window");
                let body = document.body().expect("document should have a body");
                let canvas_elem: web_sys::Element = canvas.into();
                let _ = body.append_child(&canvas_elem);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // init window
            let size = winit::dpi::LogicalSize::new(1024.0, 768.0);
            let attrs = winit::window::Window::default_attributes()
                .with_title("3D City Visualizer")
                .with_inner_size(size);
            let window = event_loop
                .create_window(attrs)
                .expect("Failed to create window");

            // For web, make sure the canvas has a real size before configuring the surface.
            #[cfg(target_arch = "wasm32")]
            Self::setup_canvas_size(&window);

            // build state
            let window_ptr = std::sync::Arc::new(window);
            let state = State::new(window_ptr.clone(), self.config.network.take());
            let state = Some(pollster::block_on(state));
            self.state = state;
            self.window = Some(window_ptr);

            #[cfg(target_arch = "wasm32")]
            self.attach_canvas();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref mut s) = self.state {
            if s.handle_event(&event) {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut s) = self.state {
                    s.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(ref mut s) = self.state {
                    s.update();
                    match s.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            s.resize(s.size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            event_loop.exit();
                        }
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}
