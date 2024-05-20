use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys::HtmlCanvasElement;

use crate::app::App;

#[cfg(not(target_arch = "wasm32"))]
use pollster;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use crate::app::State;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<'a>(
    window_size: winit::dpi::PhysicalSize<u32>, 
    init: fn(&mut App) -> Box<dyn State>, 
    render: fn(&mut App, &mut Box<dyn State>)
) {
    use crate::app::App;
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
    init: fn(&mut App) -> Box<dyn State + 'static>, 
    render: fn(&mut App, &mut Box<dyn State + 'static>)
) {
    use crate::app::App;

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::from_canvas(canvas);

    app.set_init_function(init);
    app.set_render_function(render);

    event_loop.run_app(&mut app).unwrap();
}