use std::time::Instant;
use sursface::wgpu::Buffer;
use sursface::{app::App, wgpu};
use wgpu::{util::DeviceExt, Color, CommandEncoder, RenderPipeline, ShaderModule, SurfaceTexture, TextureFormat, TextureView};
use cgmath::{Deg, Matrix4, Point3, Rad, Vector3, perspective};
use cgmath::SquareMatrix;
use bytemuck::{Pod, Zeroable};
use sursface::app::State;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    model_view_proj: [[f32; 4]; 4],
}

impl From<Matrix4<f32>> for Uniforms {
    fn from(m: Matrix4<f32>) -> Self {
        Uniforms {
            model_view_proj: m.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-1.0,  1.0,  1.0], color: [1.0, 1.0, 0.0] },
    
    Vertex { position: [-1.0, -1.0, -1.0], color: [1.0, 0.0, 1.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], color: [0.0, 1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], color: [0.0, 0.0, 0.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2,  2, 3, 0,
    4, 6, 5,  6, 4, 7,
    3, 2, 6,  6, 7, 3,
    0, 5, 1,  5, 0, 4,
    1, 6, 2,  6, 1, 5,
    0, 3, 7,  7, 4, 0,
];

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), init, render);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    use sursface::{start, wasm_bindgen};
    start::create_window_browser(canvas, init, render);
}

struct CubeState {
    render_pipeline: RenderPipeline,
    shader: ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    format: TextureFormat,
    view: Option<TextureView>,
    start_time: Instant,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
}

impl State for CubeState {}

impl CubeState {
    fn new(
        render_pipeline: RenderPipeline,
        shader: ShaderModule,
        pipeline_layout: wgpu::PipelineLayout,
        format: TextureFormat,
        device: &wgpu::Device,
    ) -> Self {
        let start_time = Instant::now();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms::from(Matrix4::identity())]),
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        CubeState {
            render_pipeline,
            shader,
            pipeline_layout,
            format,
            view: None,
            start_time,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
        }
    }

    fn set_view(&mut self, view: TextureView) {
        self.view = Some(view);
    }
}

fn init(app: &mut App) -> Box<dyn State> {
    use std::borrow::Cow;

    let display = app.display.as_ref().unwrap();
    let device = &display.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/shader.wgsl"))),
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
                            format: wgpu::VertexFormat::Float32x3,
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

    Box::new(CubeState::new(
        render_pipeline,
        shader,
        pipeline_layout,
        display.config.format,
        &device,
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
            let display = app.display.as_ref().unwrap();
            let state = unsafe { core::mem::transmute::<&mut Box<dyn State>, &mut Box<CubeState>>(state) };
            state.set_view(view);

            let now = Instant::now();
            let elapsed = (now - state.start_time).as_secs_f32();
            let aspect_ratio = app.display.as_ref().unwrap().config.width as f32 / app.display.as_ref().unwrap().config.height as f32;
            
            let model = Matrix4::from_angle_y(Rad(elapsed));
            log::info!("{}", elapsed);

            let view = Matrix4::look_at_rh(Point3::new(2.0, 2.0, 2.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
            let proj = perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);
            let mvp = proj * view * model;

            // Update the uniform buffer
            let queue = &display.queue;
            queue.write_buffer(&state.uniform_buffer, 0, bytemuck::cast_slice(&[Uniforms::from(mvp)]));

            let mut rpass = create_render_pass(state, &mut encoder, state.view.as_ref().unwrap(), color);
            draw_cube(&mut rpass, &state.render_pipeline, &state.uniform_bind_group, &state.vertex_buffer, &state.index_buffer, state.num_indices);
        }

        {
            let display = app.display.as_ref().unwrap();
            display.queue.submit(std::iter::once(encoder.finish()));
        }

        output
    };

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
    state: &'a CubeState,
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

pub fn draw_cube<'a>(
    rpass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_group: &'a wgpu::BindGroup,
    vertex_buffer: &'a wgpu::Buffer,
    index_buffer: &'a wgpu::Buffer,
    num_indices: u32,
) {
    rpass.set_pipeline(pipeline);
    rpass.set_bind_group(0, bind_group, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    rpass.draw_indexed(0..num_indices, 0, 0..1);
}

fn present(output: SurfaceTexture) {
    output.present();
}
