use sursface::display::Display;
use sursface::std::{clear_screen, get_framebuffer};
use sursface::time::now;
use sursface::wgpu::util::DeviceExt;
use sursface::wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsages, Color, CommandEncoder, CommandEncoderDescriptor, Device, Face, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceTexture, TextureView, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};
use sursface::winit::dpi::PhysicalPosition;
use sursface::winit::event::{ElementState, MouseButton, WindowEvent};
use sursface::wgpu;
use viewport::INDICES;
use sursface::cgmath::{Vector2, Zero};
use bytemuck::{Pod,Zeroable};

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
    use sursface::start;
    start::create_window_browser(canvas, &init, &render, &event);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

struct MandelbrotState {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    uniforms: Uniforms,
    scale_speed: f32,
    last_cursor_location: PhysicalPosition<f32>,
    cursor_location: PhysicalPosition<f32>,
    panning: bool,
    last_pressed_down: f32,
    zooming: bool,
    last_timestep: f32
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    translation: [f32; 2],
    cursor_pos: [f32; 2],
    scale: f32,
    _padding: f32,
}

fn create_uniforms(device: &Device) -> (Buffer, BindGroupLayout, BindGroup) {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[Uniforms { translation: Vector2::zero().into(), cursor_pos: Vector2::zero().into(), scale: 4f32, _padding: 0f32 }]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    (uniform_buffer, uniform_bind_group_layout, uniform_bind_group)
}

fn init(display: &mut Display) -> MandelbrotState {
    use std::borrow::Cow;

    let device = &display.device;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/shader.wgsl"))),
    });

    let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group ) = create_uniforms(device);

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        },
                        VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: VertexFormat::Float32x2,
                        },
                    ],
                },
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(display.config.format.into())],
            compilation_options: Default::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(viewport::VERTICES),
        usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(viewport::INDICES),
        usage: BufferUsages::INDEX,
    });

    MandelbrotState {
        render_pipeline,
        vertex_buffer,
        index_buffer,
        uniform_buffer,
        uniform_bind_group,
        uniforms: Uniforms { translation: Vector2::zero().into(), cursor_pos: Vector2::zero().into(), scale: 4f32, _padding: 0f32 },
        scale_speed: 1.0f32 - 0.1f32,
        last_cursor_location: PhysicalPosition::new(0f32, 0f32),
        cursor_location: PhysicalPosition::new(0f32, 0f32),
        panning: false,
        last_pressed_down: now(),
        zooming: false,
        last_timestep: now()
    }
}

fn render(display: &mut Display, state: &mut MandelbrotState) {
    let dt = now() - state.last_timestep;
    state.last_timestep = now();

    let clear_color = Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 255.0 / 255.0,
    };

    if (now() - state.last_pressed_down) > 0.5f32 {
        state.zooming = true;
    }

    if state.zooming && state.panning {
        let old_scale = state.uniforms.scale;
        state.uniforms.scale *= state.scale_speed.powf(dt as f32);

        let aspect_ratio = display.config.width as f32 / display.config.height as f32;

        let cursor_world = Vector2::new(
            state.uniforms.translation[0] + (state.cursor_location.x / display.config.width as f32 - 0.5) * old_scale * aspect_ratio,
            state.uniforms.translation[1] + (state.cursor_location.y / display.config.height as f32 - 0.5) * old_scale
        );

        state.uniforms.translation[0] = cursor_world.x - (state.cursor_location.x / display.config.width as f32 - 0.5) * state.uniforms.scale * aspect_ratio;
        state.uniforms.translation[1] = cursor_world.y - (state.cursor_location.y / display.config.height as f32 - 0.5) * state.uniforms.scale;
    }

    let output = {
        let mut encoder = display.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        let (output, view) = get_framebuffer(&display.surface);

        {
            let mut rpass = clear_screen(&view, &mut encoder, clear_color);

            state.uniforms.cursor_pos = [state.cursor_location.x / display.config.width as f32, state.cursor_location.y / display.config.height as f32];

            let queue = &display.queue;
            queue.write_buffer(&state.uniform_buffer, 0, bytemuck::cast_slice(&[state.uniforms]));

            draw_mandelbrot(&mut rpass, &state.render_pipeline, state);
        }

        display.queue.submit(std::iter::once(encoder.finish()));

        output
    };

    output.present();
}

fn draw_mandelbrot<'a>(
    rpass: &mut RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    state: &'a MandelbrotState
) {
    rpass.set_pipeline(pipeline);
    rpass.set_bind_group(0, &state.uniform_bind_group, &[]);
    rpass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
    rpass.set_index_buffer(state.index_buffer.slice(..), IndexFormat::Uint16);
    rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
}

fn event<'a>(display: &mut Display, state: &mut MandelbrotState, event: WindowEvent) {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            state.cursor_location = PhysicalPosition { x: position.x as f32, y: position.y as f32 };
            if state.panning {
                let dx = (state.cursor_location.x - state.last_cursor_location.x) / display.size.width as f32;
                let dy = (state.cursor_location.y - state.last_cursor_location.y) / display.size.height as f32;
                state.uniforms.translation[0] -= dx;
                state.uniforms.translation[1] += dy;
            }
            state.last_cursor_location = state.cursor_location;
        }
        WindowEvent::MouseWheel { delta, .. } => {
            if let sursface::winit::event::MouseScrollDelta::PixelDelta(delta) = delta {
                let scale_change = if delta.y > 0.0 { 0.9 } else { 1.1 };
                state.scale_speed = scale_change;
                state.zooming = true;
            }
        }
        WindowEvent::MouseInput { state: elem_state, button, .. } => {
            if elem_state == ElementState::Pressed && button == MouseButton::Left {
                state.panning = true;
                state.last_pressed_down = now();
                state.last_cursor_location = state.cursor_location;
            } else if elem_state == ElementState::Released && button == MouseButton::Left {
                state.panning = false;
                state.zooming = false;
            }
        }
        _ => {}
    }
}
