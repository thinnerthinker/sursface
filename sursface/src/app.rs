use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use super::display::Display;

pub struct App<'a, State> {
    pub display: Option<Display<'a>>,
    pub initial_size: PhysicalSize<u32>,
    #[cfg(target_arch = "wasm32")]
    pub canvas: wgpu::web_sys::HtmlCanvasElement,
    pub state: Option<Arc<Mutex<State>>>,
    pub init: Arc<dyn Fn(&mut App<State>) -> State>,
    pub render: Arc<dyn Fn(&mut App<State>, &mut State)>,
    pub event: Arc<dyn Fn(&mut App<State>, &mut State, WindowEvent)>,
}


fn init_logger() {

    let base_level = log::LevelFilter::Info;
    let wgpu_level = log::LevelFilter::Error;

    fern::Dispatch::new()
        .level(base_level)
        .level_for("wgpu_core", wgpu_level)
        .level_for("wgpu_hal", wgpu_level)
        .level_for("naga", wgpu_level)
        .chain(fern::Output::call(console_log::log))
        .apply()
        .unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
}

impl<'a, State> App<'a, State> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_window_size(
        size: PhysicalSize<u32>,
        init_func: &'static dyn Fn(&mut App<State>) -> State,
        render_func: &'static dyn Fn(&mut App<State>, &mut State),
        event_func: &'static dyn Fn(&mut App<State>, &mut State, WindowEvent)) -> Self
    {
        log::debug!("Setting window size");
        App {
            initial_size: size,
            display: None,
            state: None,
            init: Arc::new(init_func),
            render: Arc::new(render_func),
            event: Arc::new(event_func)
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas(
        canvas: wgpu::web_sys::HtmlCanvasElement,
        init_func: &'static dyn Fn(&mut App<State>) -> State,
        render_func: &'static dyn Fn(&mut App<State>, &mut State),
        event_func: &'static dyn Fn(&mut App<State>, &mut State, WindowEvent)) -> Self
    {
        log::debug!("Setting canvas size");
        App {
            initial_size: PhysicalSize::new(canvas.width(), canvas.height()),
            canvas,
            display: None,
            state: None,
            init: Arc::new(init_func),
            render: Arc::new(render_func),
            event: Arc::new(event_func)
        }
    }
}

impl<'a, State> ApplicationHandler for App<'a, State> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        init_logger();
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.display = Some(Display::from_window_size(event_loop, self.initial_size));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.display = Some(Display::from_canvas(event_loop, self.canvas.clone()));
        }

        let new_state = {
            let init_fn = Arc::clone(&self.init);
            init_fn(self)
        };

        // Now, set the state with a mutable borrow
        self.state = Some(Arc::new(Mutex::new(new_state)));
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        (Arc::clone(&self.event))(self, &mut self.state.clone().unwrap().lock().unwrap(), event.clone());

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                log::debug!("Window resized: {:?}", physical_size);
                if let Some(display) = self.display.as_mut() {
                    display.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                (Arc::clone(&self.render))(self, &mut self.state.clone().unwrap().lock().unwrap());

                if let Some(display) = self.display.as_mut() {
                    display.window.as_ref().request_redraw();
                }
            }
            _ => ()
        };
    }
}