use std::panic;
use winit::{dpi::PhysicalSize, event_loop::{ControlFlow, EventLoop}};

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

fn start_event_flow() -> EventLoop<()> {
    init_logger();

    #[cfg(target_arch = "wasm32")]
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_desktop(window_size: PhysicalSize<u32>) {
    let event_loop = start_event_flow();
    let mut app = crate::app::App::from_window_size(window_size);
    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn start_browser(canvas: wgpu::web_sys::HtmlCanvasElement) {
    let event_loop = start_event_flow();
    let mut app = crate::app::App::from_canvas(canvas);
    event_loop.run_app(&mut app).unwrap();
}
