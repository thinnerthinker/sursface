use sursface::{app::{App, State}, wgpu};
use std::any::Any;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    env_logger::init();
    log::info!("Starting application");
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), init, render);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[derive(Clone)]
struct EmptyState {}
impl State for EmptyState {}

fn init<'a>(app: &mut App<'a>) -> Box<dyn State> {
    log::info!("Initializing state");
    Box::new(EmptyState {})
}

fn render<'a>(app: &mut App<'a>, _state: &mut Box<dyn State>) {
    log::error!("hhhhom");
    let output = clear_screen(app, wgpu::Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 1.0,
    }).unwrap();
    let _ = present(app, output);
}


fn clear_screen<'a>(app: &mut App<'a>, color: sursface::wgpu::Color) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
    log::info!("gaspar");
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

fn present<'a>(app: &mut App<'a>, output: sursface::wgpu::SurfaceTexture) -> Result<(), wgpu::SurfaceError> {
    output.present();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}