use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys::HtmlCanvasElement;

use crate::app::App;
use crate::app::AppState;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<State: AppState + 'static>(window_size: winit::dpi::PhysicalSize<u32>)
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<State>::from_window_size(window_size);
    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn create_window_browser<State: AppState + 'static>(canvas: HtmlCanvasElement) 
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<State>::from_canvas(canvas);
    event_loop.run_app(&mut app).unwrap();
}