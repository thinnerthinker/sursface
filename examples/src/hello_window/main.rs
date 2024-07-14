use sursface::app::AppState;
use sursface::display::Display;
use sursface::wgpu::{self, TextureView};
use sursface::winit::dpi::PhysicalSize;

#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    sursface::start::create_window_desktop::<EmptyState>(PhysicalSize::new(1280, 720));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    sursface::start::create_window_browser::<EmptyState>(canvas);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[derive(Clone)]
struct EmptyState {}

impl AppState for EmptyState {
    fn new<'a>(_display: &mut Display) -> EmptyState {
        EmptyState {}
    }

    fn draw<'a>(&mut self, display: &mut Display) {
        let output = display.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        clear_screen(
            display,
            &view,
            wgpu::Color {
                r: 100.0 / 255.0,
                g: 149.0 / 255.0,
                b: 237.0 / 255.0,
                a: 1.0,
            },
        );

        output.present();
    }
}

fn clear_screen<'a>(display: &mut Display, view: &TextureView, color: sursface::wgpu::Color) {
    let mut encoder = display
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Screen Encoder"),
        });

    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    display.queue.submit(std::iter::once(encoder.finish()));
}
