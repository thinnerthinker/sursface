use sursface::display::Display;
use sursface::std::models::{quad_uvs, cube, VertexPositionNormalUv};
use sursface::std::{clear_screen, create_render_pipeline, create_sampler_entry, create_shader, create_texture, create_texture_layout_entry_from_image, create_uniforms, get_framebuffer};
use sursface::time::now_secs;
use sursface::wgpu::{BindGroupEntry, Buffer};
use sursface::winit::event::WindowEvent;
use sursface::wgpu;
use wgpu::{util::DeviceExt, Color, RenderPipeline};
use sursface::cgmath::{Deg, Matrix4, Point3, Vector3, perspective};
use sursface::cgmath::SquareMatrix;
use sursface::{app::AppState, winit::dpi::PhysicalSize};

#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    sursface::start::create_window_desktop::<CubeState>(PhysicalSize::new(1280, 720));
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    sursface::start::create_window_browser::<CubeState>(canvas);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

struct CubeState {
    render_pipeline: RenderPipeline,
    start_time: f32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: Buffer,
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

impl AppState for CubeState {
    fn init(display: &mut Display) -> CubeState {
        let device = &display.device;

        let shader = create_shader(device, include_str!("assets/shader.wgsl"));

        let (texture_bind_group_entry, texture_view) = create_texture_layout_entry_from_image(device, &display.queue, include_bytes!("assets/dice.png"), 0);
        let (sampler_entry, sampler) = create_sampler_entry(device, 1);

        let (texture_bind_group_layout,texture_bind_group ) = create_texture(device, 
            &[texture_bind_group_entry, sampler_entry],
            &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }, BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }]);
        
        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group ) = create_uniforms(device, 
            Uniforms { model_view_proj: Matrix4::identity().into(), camera_pan: Matrix4::identity().into() },
            0);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(display, pipeline_layout, shader, &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<VertexPositionNormalUv>() as wgpu::BufferAddress,
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
                    wgpu::VertexAttribute {
                        offset: 24,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            },
        ]);

        let start_time = now_secs();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube(&[
                quad_uvs((0.00, 1f32 / 3f32), (0.25 + 0.00, 2f32 / 3f32)), // 6
                quad_uvs((0.50, 1f32 / 3f32), (0.25 + 0.50, 2f32 / 3f32)), // 1
                quad_uvs((0.25, 1f32 / 3f32), (0.25 + 0.25, 2f32 / 3f32)), // 4
                quad_uvs((0.75, 1f32 / 3f32), (0.25 + 0.75, 2f32 / 3f32)), // 3
                quad_uvs((0.50, 0f32 / 3f32), (0.25 + 0.50, 1f32 / 3f32)), // 2
                quad_uvs((0.50, 2f32 / 3f32), (0.25 + 0.50, 3f32 / 3f32)), // 5
            ])),
            usage: wgpu::BufferUsages::VERTEX,
        });

        CubeState {
            render_pipeline,
            start_time,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            texture_bind_group,
            uniforms: Uniforms { model_view_proj: Matrix4::identity().into(), camera_pan: Matrix4::identity().into() },
            yaw: 0f64,
            pitch: 0f64,
            pan_speed: 0.4f64
        }
    }

    fn render(&mut self, display: &mut Display) {
        let clear_color = Color {
            r: 252.0 / 255.0,
            g: 241.0 / 255.0,
            b: 139.0 / 255.0,
            a: 255.0 / 255.0,
        };

        let output = {
            let mut encoder = display.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

            let (output, view) = get_framebuffer(&display.surface);
            {
                let mut rpass = clear_screen(&view, &mut encoder, clear_color);

                let now = now_secs();
                let elapsed = now - self.start_time;
                sursface::log::info!("{} {}", now, elapsed);
                let aspect_ratio = display.config.width as f32 / display.config.height as f32;
                
                let model = Matrix4::identity();

                let view = Matrix4::look_at_rh(Point3::new(3.0, 3.0, 3.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
                let proj = perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);
                let mvp = proj * view * model;

                self.uniforms.model_view_proj = mvp.into();

                let queue = &display.queue;
                queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));

                {
                    rpass.set_pipeline(&self.render_pipeline);
                    rpass.set_bind_group(0,  &self.uniform_bind_group, &[]);
                    rpass.set_bind_group(1, &self.texture_bind_group, &[]);
                    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    rpass.draw(0..36, 0..1);
                }
            }

            {
                display.queue.submit(std::iter::once(encoder.finish()));
            }

            output
        };

        output.present();
    }

    fn event<'a>(&mut self, _display: &mut Display, event: WindowEvent) {
        let mut x = 0f64;
        let mut y = 0f64;
        
        let moved = {
            match event {
                WindowEvent::Touch(a) => {
                    x = a.location.x;
                    y = a.location.y;

                    true
                }
                WindowEvent::CursorMoved { device_id: _, position } => {
                    x = position.x;
                    y = position.y;

                    true
                }
                _ => false
            }
        };

        if moved {
            self.yaw = x;
            self.pitch = -y;
            
            self.uniforms.camera_pan = (Matrix4::from_angle_y(Deg(self.yaw * self.pan_speed)) * 
                Matrix4::from_angle_x(Deg(self.pitch * self.pan_speed)))
                .cast().unwrap().into();
        }
    }
}