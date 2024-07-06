use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys::HtmlCanvasElement;

use crate::app::App;
use crate::app::AppHandlers;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<State: 'static>(
    window_size: winit::dpi::PhysicalSize<u32>, 
    handlers: AppHandlers<State>)
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::from_window_size(window_size, handlers);
    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn create_window_browser<State: 'static>(
    canvas: HtmlCanvasElement, 
    handlers: AppHandlers<State>) 
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::from_canvas(canvas, handlers);
    event_loop.run_app(&mut app).unwrap();
}