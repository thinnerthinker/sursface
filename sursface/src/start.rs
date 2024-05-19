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


#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<'a>(window_size: PhysicalSize<u32>, render: fn(&mut App<'a>)) {
    use crate::app::App;

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::from_window_size(window_size);
    app.set_render_function(render);

    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn create_window_browser<'a>(canvas: HtmlCanvasElement, render: fn(&mut App<'a>)) {
    use crate::app::App;

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::from_canvas(canvas);
    app.set_render_function(render);

    event_loop.run_app(app).unwrap();
}