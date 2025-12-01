use crate::state::State;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App {
    state: Option<State>,
    window: Option<std::sync::Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            window: None,
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

            // build state
            let window_ptr = std::sync::Arc::new(window);
            let state = State::new(window_ptr.clone());
            let state = Some(pollster::block_on(state));
            self.state = state;
            self.window = Some(window_ptr);
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
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref w) = self.window {
            w.request_redraw();
        }
    }
}
