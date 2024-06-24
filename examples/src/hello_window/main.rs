use sursface::{app::App, wgpu};
#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), &init, &render);
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    use sursface::{start, wasm_bindgen};
    start::create_window_browser(canvas, &init, &render);
}

#[cfg(target_arch = "wasm32")]
fn main() {}


#[derive(Clone)]
struct EmptyState {}

fn init<'a>(_app: &mut App<EmptyState>) -> EmptyState {
    EmptyState {}
}

fn render<'a>(app: &mut App<EmptyState>, _state: &mut EmptyState) {
    let output = clear_screen(app, wgpu::Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 1.0,
    }).unwrap();
    let _ = present(app, output);
}


fn clear_screen<'a>(app: &mut App<EmptyState>, color: sursface::wgpu::Color) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
    let display = app.display.as_ref().unwrap();
    let output = display.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = display.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

        Ok(output)
}

fn present<'a>(app: &mut App<EmptyState>, output: sursface::wgpu::SurfaceTexture) -> Result<(), wgpu::SurfaceError> {
    output.present();
    Ok(())
}