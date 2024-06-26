use sursface::{display::Display, std::{clear_screen, get_framebuffer}, wgpu, winit::event::WindowEvent};

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
    let clear_color = wgpu::Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 1.0,
    };
    
    let mut encoder = display.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Encoder"),
    });

    let (output, view) = get_framebuffer(&display.surface);
    {
        let mut _rpass = clear_screen(&view, &mut encoder, clear_color);
    }

    output.present();
}

fn event<'a>(_display: &mut Display, _state: &mut EmptyState, _event: WindowEvent) {}