use sursface::wgpu::{BindGroup, BindGroupLayout, Buffer, Device, FragmentState, PipelineCompilationOptions, Queue, RenderPass, Surface, VertexBufferLayout, VertexState};
use sursface::winit::dpi::PhysicalPosition;
use sursface::winit::event::{ElementState, MouseButton, WindowEvent};
use sursface::{app::App, wgpu};
use viewport::INDICES;
use wgpu::{util::DeviceExt, Color, CommandEncoder, RenderPipeline, SurfaceTexture, TextureView};
use cgmath::{perspective, Deg, Matrix4, Point3, Rad, Transform, Vector2, Vector3, Zero};
use cgmath::SquareMatrix;
use image::{GenericImageView, ImageFormat};

#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

use crate::viewport::Vertex;

pub mod viewport;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(720, 720), &init, &render, &event);
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    use sursface::{start, wasm_bindgen};
    start::create_window_browser(canvas, &init, &render, &event);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

struct MandelbrotState {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    translation: Vector2<f64>,
    scale: f64,
    uniforms: Uniforms,
    translation_speed: f64,
    scale_speed: f64,
    last_cursor_location: PhysicalPosition<f64>,
    cursor_location: PhysicalPosition<f64>,
    panning: bool
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    model_view_proj: [[f32; 4]; 4]
}

fn create_uniforms(device: &Device) -> (Buffer, BindGroupLayout, BindGroup) {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[Uniforms { model_view_proj: Matrix4::identity().into() }]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    (uniform_buffer, uniform_bind_group_layout, uniform_bind_group)
}

fn init(app: &mut App<MandelbrotState>) -> MandelbrotState {
    use std::borrow::Cow;

    let display = app.display.as_ref().unwrap();
    let device = &display.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/shader.wgsl"))),
    });

    let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group ) = create_uniforms(device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                },
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(display.config.format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(viewport::VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(viewport::INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    MandelbrotState {
        render_pipeline,
        vertex_buffer,
        index_buffer,
        uniform_buffer,
        uniform_bind_group,
        uniforms: Uniforms { model_view_proj: Matrix4::identity().into() },
        translation: Vector2::zero(),
        scale: 1f64,
        translation_speed: 0.0001f64,
        scale_speed: 0.1f64,
        last_cursor_location: PhysicalPosition::new(0f64, 0f64),
        cursor_location: PhysicalPosition::new(0f64, 0f64),
        panning: false
    }
}

fn render(app: &mut App<MandelbrotState>, state: &mut MandelbrotState) {
    let clear_color = Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 255.0 / 255.0,
    };

    let output = {
        let display = app.display.as_ref().unwrap();

        let mut encoder = display.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        let (output, view) = get_framebuffer(&display.surface);
        
        {
            let mut rpass = clear_screen(&view, &mut encoder, clear_color);

            let display = app.display.as_ref().unwrap();
            let aspect_ratio = app.display.as_ref().unwrap().config.width as f32 / app.display.as_ref().unwrap().config.height as f32;

            let mvp = Matrix4::from_scale(state.scale) *
                Matrix4::from_translation(Vector3::<f64>::new(state.translation.x, state.translation.y, 0f64));
            state.uniforms.model_view_proj = mvp.cast().unwrap().into();

            let queue = &display.queue;
            queue.write_buffer(&state.uniform_buffer, 0, bytemuck::cast_slice(&[state.uniforms]));

            draw_mandelbrot(&mut rpass, &state.render_pipeline, state);
        }

        {
            let display = app.display.as_ref().unwrap();
            display.queue.submit(std::iter::once(encoder.finish()));
        }

        output
    };

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


fn draw_mandelbrot<'a>(
    rpass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    state: &'a MandelbrotState
) {
    rpass.set_pipeline(pipeline);
    rpass.set_bind_group(0, &state.uniform_bind_group, &[]);
    rpass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
    rpass.set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
}


fn event<'a>(app: &mut App<MandelbrotState>, state: &mut MandelbrotState, event: WindowEvent) {
    let moved = {
        match event {
            WindowEvent::Touch(a) => {
                state.cursor_location = PhysicalPosition { x: a.location.x, y: a.location.y };
                true
            }
            WindowEvent::CursorMoved { device_id, position } => {
                state.cursor_location  = position;
                true
            }
            _ => false
        }
    };

    let mut scale = 0f64;
    let scaled = {
        match event {
            WindowEvent::MouseWheel { device_id, delta, phase } => {
                match delta {
                    sursface::winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x: _, y: d }) => {
                        scale = d;
                        true
                    }
                    _ => false
                }
            }
            _ => false
        }
    };

    match event {
        WindowEvent::MouseInput { device_id, state: elem_state, button } => {
            if elem_state == ElementState::Pressed && button == MouseButton::Left {
                state.panning = true;
            } else if elem_state == ElementState::Released && button == MouseButton::Left {
                state.panning = false;
            }
        }
        _ => ()
    }

    if moved && state.panning {
        state.translation.x += (state.cursor_location.x - state.last_cursor_location.x) * state.translation_speed;
        state.translation.y -= (state.cursor_location.y - state.last_cursor_location.y) * state.translation_speed;
    }

    state.last_cursor_location = state.cursor_location;

    if scaled {
        state.scale += scale * state.scale_speed;
    }

    
}