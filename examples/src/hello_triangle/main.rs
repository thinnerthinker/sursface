use std::any::Any;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use sursface::app::{App, State};
use sursface::wgpu::{self, Color, CommandEncoder, RenderPipeline, ShaderModule, SurfaceTexture, TextureFormat, TextureView};

// Define the State trait


// Implement State for TriangleState
struct TriangleState {
    render_pipeline: RenderPipeline,
    shader: ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    format: TextureFormat,
    view: Option<TextureView>,
}

impl State for TriangleState {}

impl TriangleState {
    fn new(
        render_pipeline: RenderPipeline,
        shader: ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        format: TextureFormat,
    ) -> Self {
        TriangleState {
            render_pipeline,
            shader,
            pipeline_layout,
            format,
            view: None,
        }
    }

    fn set_view(&mut self, view: TextureView) {
        self.view = Some(view);
    }
}

fn init(app: &mut App) -> Box<dyn State> {
    let display = app.display.as_ref().unwrap();
    let device = &display.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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

    Box::new(TriangleState::new(
        render_pipeline,
        shader,
        pipeline_layout,
        display.config.format,
    ))
}

fn render(app: &mut App, mut state: &mut Box<dyn State>) {
    let color = Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 255.0 / 255.0,
    };

    let output = {
        let (output, mut encoder, view) = {
            let display = app.display.as_ref().unwrap();

            // Clear the screen and get the output, encoder, and view
            let (output, mut encoder) = clear_screen(app, color).unwrap();
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            (output, encoder, view)
        };

        {
            let state = unsafe { core::mem::transmute::<&mut Box<dyn State>, &mut Box<TriangleState>>(state) };
            state.set_view(view);

            let mut rpass = create_render_pass(state, &mut encoder, state.view.as_ref().unwrap(), color);
            draw_triangle(&mut rpass, &state.render_pipeline);
        }

        // Submit the queue outside of mutable borrow of state
        {
            let display = app.display.as_ref().unwrap();
            display.queue.submit(std::iter::once(encoder.finish()));
        }

        output
    };

    // Now perform the present operation after mutable borrows are done
    present(output);
}

fn clear_screen(
    app: &mut App,
    color: Color,
) -> Result<(SurfaceTexture, CommandEncoder), wgpu::SurfaceError> {
    let display = app.display.as_ref().unwrap();
    let output = display.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = display.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Clear Screen Encoder"),
    });

    // Clear the screen with the specified color
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Screen Render Pass"),
            timestamp_writes: Default::default(),
            occlusion_query_set: Default::default(),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
        });
        // No further commands needed for clearing
    }

    Ok((output, encoder))
}

fn create_render_pass<'a>(
    state: &'a TriangleState,
    encoder: &'a mut CommandEncoder,
    view: &'a TextureView,
    color: Color,
) -> wgpu::RenderPass<'a> {
    let rpass_descriptor = wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
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

fn present(output: SurfaceTexture) {
    output.present();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), init, render);
}

#[cfg(target_arch = "wasm32")]
fn main() {}
