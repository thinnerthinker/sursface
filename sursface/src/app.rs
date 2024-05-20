use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;
use std::any::Any;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use super::display::Display;

pub trait State {}

#[derive(Default)]
pub struct App<'a> {
    pub display: Option<Display<'a>>,
    pub initial_size: PhysicalSize<u32>,
    #[cfg(target_arch = "wasm32")]
    pub canvas: Option<wgpu::web_sys::HtmlCanvasElement>,
    pub state: Option<Arc<Mutex<Box<dyn State>>>>,
    pub init: Option<Box<dyn Fn(&mut App<'a>) -> Box<dyn State + 'static>>>,
    pub render: Option<Box<dyn Fn(&mut App<'a>, &mut Box<dyn State + 'static>)>>,
}


fn init_logger() {
    #[cfg(target_arch = "wasm32")] 
    {
        // We keep wgpu at Error level, as it's very noisy.
        let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;

        // On web, we use fern, as console_log doesn't have filtering on a per-module level.
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;
        
        // parse_default_env will read the RUST_LOG environment variable and apply it on top
        // of these default filters.
        env_logger::builder()
            .filter_level(base_level)
            .filter_module("wgpu_core", wgpu_level)
            .filter_module("wgpu_hal", wgpu_level)
            .filter_module("naga", wgpu_level)
            .parse_default_env()
            .init();
    }
}

impl<'a> App<'a> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_window_size(size: PhysicalSize<u32>) -> Self {
        log::info!("Setting window size");
        App {
            initial_size: size,
            ..Default::default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas(canvas: wgpu::web_sys::HtmlCanvasElement) -> Self {
        log::info!("Setting canvas size");
        App {
            initial_size: PhysicalSize::new(canvas.width(), canvas.height()),
            canvas: Some(canvas),
            ..Default::default()
        }
    }

    pub fn set_init_function<F>(&mut self, init_func: F)
    where
        F: Fn(&mut App<'a>) -> Box<dyn State + 'static> + 'static,
    {
        self.init = Some(Box::new(init_func));
    }

    pub fn set_render_function<F>(&mut self, render_func: F)
    where
        F: Fn(&mut App<'a>, &mut Box<dyn State + 'static>) + 'static,
    {
        self.render = Some(Box::new(render_func));
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        init_logger();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.display = Some(Display::from_window_size(event_loop, self.initial_size));
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(canvas) = self.canvas.clone() {
                self.display = Some(Display::from_canvas(event_loop, canvas));
            } else {
                log::error!("Canvas is not set");
            }
        }

        if let Some(init) = self.init.take() {
            self.state = Some(Arc::new(Mutex::new(init(self))));
            self.init = Some(init);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
                log::info!("Window resized: {:?}", physical_size);
                if let Some(display) = self.display.as_mut() {
                    display.resize(physical_size);
                }

                if let Some(init) = self.init.take() {
                    self.state = Some(Arc::new(Mutex::new(init(self))));
                    self.init = Some(init);
                }
            }
            WindowEvent::RedrawRequested => {
                log::error!("Redraw requested");
                log::error!("Before rendering");

                if let Some(render) = self.render.take() {
                    log::error!("Locking state");
                    let state_arc = Arc::clone(self.state.as_ref().unwrap());
                    let mut state_lock = state_arc.lock().unwrap();
                    log::error!("State locked");

                    render(self, &mut *state_lock);
                }

                log::error!("After rendering");
            }
            _ => (),
        }
    }
}