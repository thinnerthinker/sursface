use sursface::{app::App, display::{self, Display}, std::{clear_screen, create_render_pipeline, create_shader, get_framebuffer}, wgpu::{self, Color, CommandEncoder, RenderPass, RenderPipeline, Surface, SurfaceTexture, TextureView}, winit::event::WindowEvent};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::{dpi::PhysicalSize, event};
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), &init, &render, &event);
}

#[cfg(target_arch = "wasm32")]
fn main() {}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    use sursface::start;

    start::create_window_browser(canvas, &init, &render, &event);
}

struct TriangleState {
    render_pipeline: RenderPipeline,
}

fn init(display: &mut Display) -> TriangleState {
    use std::borrow::Cow;
    
    let device = &display.device;

    let shader = create_shader(device, include_str!("assets/shader.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    
    let render_pipeline = create_render_pipeline(display, pipeline_layout, shader, &[]);
    TriangleState { render_pipeline }
}

fn render(display: &mut Display, state: &mut TriangleState) {
    let clear_color = Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 255.0 / 255.0,
    };

    let device = &display.device;

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Encoder"),
    });

    let (output, view) = get_framebuffer(&display.surface);
    {
        let mut rpass = clear_screen(&view, &mut encoder, clear_color);
        draw_triangle(&mut rpass, &state.render_pipeline);
    }

    display.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

pub fn draw_triangle<'a>(
    rpass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
) {
    rpass.set_pipeline(pipeline);
    rpass.draw(0..3, 0..1);
}

fn event<'a>(_display: &mut Display, _state: &mut TriangleState, _event: WindowEvent) {}