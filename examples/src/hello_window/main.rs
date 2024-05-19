use sursface::{app::App, wgpu};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), render);
}

fn render<'a>(app: &mut App<'a>) {
    let _ = clear_screen(app, sursface::wgpu::Color { r: 100f64 / 255f64, g: 149f64 / 255f64, b: 237f64 / 255f64, a: 255f64 / 255f64 });
}

fn clear_screen<'a>(app: &mut App<'a>, color: sursface::wgpu::Color) -> Result<(), wgpu::SurfaceError> {
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
        output.present();

        Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}