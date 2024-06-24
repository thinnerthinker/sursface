use std::time::Instant;
use cube::{INDICES, VERTICES};
use sursface::wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPass, Surface};
use sursface::winit::event::WindowEvent;
use sursface::{app::App, wgpu};
use wgpu::{util::DeviceExt, Color, CommandEncoder, RenderPipeline, SurfaceTexture, TextureView};
use cgmath::{Deg, Matrix4, Point3, Rad, Vector3, perspective};
use cgmath::SquareMatrix;
use image::{GenericImageView, ImageFormat};

use crate::cube::Vertex;

mod cube;

#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use sursface::winit::dpi::PhysicalSize;
    sursface::start::create_window_desktop(PhysicalSize::new(1280, 720), &init, &render, &event);
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    use sursface::{start, wasm_bindgen};
    start::create_window_browser(canvas, &init, &render, &event);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

struct CubeState {
    render_pipeline: RenderPipeline,
    start_time: Instant,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    texture_bind_group: wgpu::BindGroup,
    uniforms: Uniforms,
    yaw: f64,
    pitch: f64,
    pan_speed: f64
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    model_view_proj: [[f32; 4]; 4],
    camera_pan: [[f32; 4]; 4],
}

fn load_texture(device: &Device, queue: &Queue) -> (BindGroupLayout, BindGroup) {
    let image_bytes = include_bytes!("assets/dice.png");
    let img = image::load(std::io::Cursor::new(image_bytes.as_ref()), ImageFormat::Png).unwrap();
    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();

    let texture_extent = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_extent,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("Texture Bind Group"),
    });

    (texture_bind_group_layout, texture_bind_group)
}

fn create_uniforms(device: &Device) -> (Buffer, BindGroupLayout, BindGroup) {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[Uniforms { model_view_proj: Matrix4::identity().into(), camera_pan: Matrix4::identity().into() }]),
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

fn init(app: &mut App<CubeState>) -> CubeState {
    use std::borrow::Cow;

    let display = app.display.as_ref().unwrap();
    let device = &display.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/shader.wgsl"))),
    });

    let (texture_bind_group_layout, texture_bind_group) = load_texture(device, &display.queue);
    let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group ) = create_uniforms(device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
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

    let start_time = Instant::now();

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
        start_time,
        uniform_buffer,
        uniform_bind_group,
        vertex_buffer,
        index_buffer,
        num_indices: 36,
        texture_bind_group,
        uniforms: Uniforms { model_view_proj: Matrix4::identity().into(), camera_pan: Matrix4::identity().into() },
        yaw: 0f64,
        pitch: 0f64,
        pan_speed: 0.3f64
    }
}

fn render(app: &mut App<CubeState>, state: &mut CubeState) {
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

            let now = Instant::now();
            let elapsed = (now - state.start_time).as_secs_f32();
            let aspect_ratio = app.display.as_ref().unwrap().config.width as f32 / app.display.as_ref().unwrap().config.height as f32;
            
            let model = Matrix4::from_angle_y(Rad(elapsed));
            log::debug!("{}", elapsed);

            let view = Matrix4::look_at_rh(Point3::new(3.0, 3.0, 3.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
            let proj = perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);
            let mvp = proj * view * model;

            state.uniforms.model_view_proj = mvp.into();

            let queue = &display.queue;
            queue.write_buffer(&state.uniform_buffer, 0, bytemuck::cast_slice(&[state.uniforms]));

            draw_cube(&mut rpass, &state.render_pipeline, &state.uniform_bind_group, &state.vertex_buffer, &state.index_buffer, state.num_indices, &state.texture_bind_group);
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

pub fn draw_cube<'a>(
    rpass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_group: &'a wgpu::BindGroup,
    vertex_buffer: &'a wgpu::Buffer,
    index_buffer: &'a wgpu::Buffer,
    num_indices: u32,
    texture_bind_group: &'a wgpu::BindGroup,
) {
    rpass.set_pipeline(pipeline);
    rpass.set_bind_group(0, bind_group, &[]);
    rpass.set_bind_group(1, texture_bind_group, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    rpass.draw_indexed(0..num_indices, 0, 0..1);
}


fn event<'a>(app: &mut App<CubeState>, state: &mut CubeState, event: WindowEvent) {
    let mut x = 0f64;
    let mut y = 0f64;
    
    let moved = {
        match event {
            WindowEvent::Touch(a) => {
                x = a.location.x;
                y = a.location.y;

                true
            }
            WindowEvent::CursorMoved { device_id, position } => {
                x = position.x;
                y = position.y;

                true
            }
            _ => false
        }
    };

    if moved {
        state.yaw = (x - state.yaw) * state.pan_speed;
        state.pitch = -(y - state.pitch) * state.pan_speed;
        
        state.uniforms.camera_pan = (Matrix4::from_angle_y(Deg(state.yaw)) * Matrix4::from_angle_x(Deg(state.pitch)))
            .cast().unwrap().into();
    }
}