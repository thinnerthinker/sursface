use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys::HtmlCanvasElement;

use crate::app::App;
use crate::display::Display;
use winit::event::WindowEvent;

#[cfg(not(target_arch = "wasm32"))]
use pollster;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<State: 'static>(
    window_size: winit::dpi::PhysicalSize<u32>, 
    init: &'static dyn Fn(&mut Display) -> State,
    render: &'static dyn Fn(&mut Display, &mut State),
    event: &'static dyn Fn( &mut Display, &mut State, WindowEvent))
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::from_window_size(window_size, init, render, event);
    event_loop.run_app(&mut app).unwrap();
}


#[cfg(target_arch = "wasm32")]
pub fn create_window_browser<State: 'static>(
    canvas: HtmlCanvasElement, 
    init: &'static dyn Fn(&mut Display) -> State,
    render: &'static dyn Fn(&mut Display, &mut State),
    event: &'static dyn Fn( &mut Display, &mut State, WindowEvent))
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::from_canvas(canvas, init, render, event);
    event_loop.run_app(&mut app).unwrap();
}