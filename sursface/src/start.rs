use std::panic;
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::app::App;

#[cfg(not(target_arch = "wasm32"))]
use pollster;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use super::display::Display;
use crate::app::State;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<'a>(
    window_size: PhysicalSize<u32>, 
    init: fn(&mut App) -> Box<dyn State>, 
    render: fn(&mut App, &mut Box<dyn State>)
) {
    use crate::app::App;

    log::error!("stating");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::from_window_size(window_size);
    log::error!("megbuilt");
    app.set_init_function(init);
    app.set_render_function(render);
    log::error!("ayayyayay");

    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn create_window_browser<'a>(
    canvas: HtmlCanvasElement, 
    init: fn(&mut App) -> Box<dyn State + 'a>, 
    render: fn(&mut App, Box<dyn State + 'a>)
) {
    use crate::app::App;

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::from_canvas(canvas);
    app.set_init_function(init);
    app.set_render_function(render);

    event_loop.run_app(&mut app).unwrap();
}