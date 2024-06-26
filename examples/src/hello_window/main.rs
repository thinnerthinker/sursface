use sursface::{display::Display, wgpu, winit::event::WindowEvent};
#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), &init, &render, &event);
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    use sursface::start;
    start::create_window_browser(canvas, &init, &render, &event);
}

#[cfg(target_arch = "wasm32")]
fn main() {}


#[derive(Clone)]
struct EmptyState {}

fn init<'a>(_display: &mut Display) -> EmptyState {
    EmptyState {}
}

fn render<'a>(display: &mut Display, _state: &mut EmptyState) {
    let output = clear_screen(display, wgpu::Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 1.0,
    }).unwrap();
    output.present();
}

fn event<'a>(_display: &mut Display, _state: &mut EmptyState, _event: WindowEvent) {}