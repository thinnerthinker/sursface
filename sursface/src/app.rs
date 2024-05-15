use std::panic;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use std::sync::{Arc,RwLock};

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use super::display::{Display};

#[derive(Default)]
pub struct App<'a> {
    display: Option<Display<'a>>,
    initial_size: PhysicalSize<u32>,
    #[cfg(target_arch = "wasm32")]
    canvas: Option<wgpu::web_sys::HtmlCanvasElement>,
}

impl<'a> App<'a> {
    pub fn from_window_size(size: PhysicalSize<u32>) -> Self {
        log::info!("setting size");

        App { 
            initial_size: size,
            ..Default::default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas(canvas: wgpu::web_sys::HtmlCanvasElement) -> Self {
        log::info!("setting canvas");

        App {
            initial_size: PhysicalSize::new(canvas.width(), canvas.height()),
            canvas: Some(canvas),
            ..Default::default()
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.display = Some(Display::from_window_size(event_loop, self.initial_size));
        };
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(canvas) = self.canvas.clone() {
                self.display = Some(Display::from_canvas(event_loop, canvas));
            } else {
                log::error!("Canvas is not set");
            }
        };
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
                log::info!("physical_size: {physical_size:?}");
                self.display.as_mut().unwrap().resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                let display = self.display.as_mut().unwrap();
                display.window.request_redraw();

                match display.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        display.resize(display.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                };
            }
            _ => (),
        }
    }
}