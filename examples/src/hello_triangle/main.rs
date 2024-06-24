use sursface::{app::App, wgpu::{self, Color, CommandEncoder, RenderPass, RenderPipeline, Surface, SurfaceTexture, TextureView}, winit::event::WindowEvent};

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

fn init(app: &mut App<TriangleState>) -> TriangleState {
    use std::borrow::Cow;
    
    let display = app.display.as_ref().unwrap();
    let device = &display.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(display.config.format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    TriangleState { render_pipeline }
}

fn render(app: &mut App<TriangleState>, state: &mut TriangleState) {
    let clear_color = Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 255.0 / 255.0,
    };

    let display = app.display.as_ref().unwrap();

    let mut encoder = display.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

fn get_framebuffer(surface: &Surface) -> (SurfaceTexture, TextureView) {
    let output = surface.get_current_texture().unwrap();
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    (output, view)
}

fn clear_screen<'a>(
    framebuffer_view: &'a TextureView,
    encoder: &'a mut CommandEncoder,
    color: Color,
) -> RenderPass<'a> {
    let rpass_descriptor = wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: framebuffer_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: Default::default(),
        occlusion_query_set: Default::default(),
    };

    encoder.begin_render_pass(&rpass_descriptor)
}

pub fn draw_triangle<'a>(
    rpass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
) {
    rpass.set_pipeline(pipeline);
    rpass.draw(0..3, 0..1);
}


fn event<'a>(_app: &mut App<TriangleState>, _state: &mut TriangleState, _event: WindowEvent) {}