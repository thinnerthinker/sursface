use sursface::display::Display;
use sursface::std::{clear_screen, create_render_pipeline, create_shader, create_uniforms, get_framebuffer};
use sursface::std::models::{quad_no_normal, quad_uvs, VertexPositionUv};
use sursface::time::now_secs;
use sursface::wgpu::util::DeviceExt;
use sursface::wgpu::{BindGroup, Buffer, BufferAddress, BufferUsages, Color, CommandEncoderDescriptor, PipelineLayoutDescriptor, RenderPass, RenderPipeline, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};
use sursface::winit::dpi::PhysicalPosition;
use sursface::winit::event::{ElementState, MouseButton, WindowEvent};
use sursface::{log, wgpu};
use sursface::cgmath::{Vector2, Zero};
use bytemuck::{Pod,Zeroable};
use std::fmt::Display as FmtDisplay;

#[cfg(target_arch = "wasm32")]
use sursface::wasm_bindgen;

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

#[derive(Clone)]
enum InteractionState {
    Idle { last_pressed_down_at: f32, pre_tap: bool },
    PanningIdle { pressed_down_at: f32, pre_tap: bool },
    Panning,
    Zooming,
    ZoomingOut
}

impl FmtDisplay for InteractionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractionState::Idle { last_pressed_down_at, pre_tap } => {
                write!(f, "Idle {{ last_pressed_down_at: {}, pre_tap: {} }}", last_pressed_down_at, pre_tap)
            }
            InteractionState::PanningIdle { pressed_down_at, pre_tap } => {
                write!(f, "PanningIdle {{ pressed_down_at: {}, pre_tap: {} }}", pressed_down_at, pre_tap)
            }
            InteractionState::Panning => write!(f, "Panning"),
            InteractionState::Zooming => write!(f, "Zooming"),
            InteractionState::ZoomingOut => write!(f, "ZoomingOut"),
        }
    }
}

struct MandelbrotState {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    uniforms: Uniforms,
    scale_speed: f32,
    last_cursor_location: PhysicalPosition<f32>,
    cursor_location: PhysicalPosition<f32>,
    last_timestep: f32,
    interaction_state: InteractionState
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    translation: [f32; 2], // 8 bytes
    cursor_pos: [f32; 2],  // 8 bytes
    scale: f32,            // 4 bytes
    aspect_ratio: f32,     // 4 bytes
    _padding: [f32; 2],    // 8 bytes to make the struct size 32 bytes
}

fn init(display: &mut Display) -> MandelbrotState {
    let device = &display.device;
    let aspect_ratio = display.config.width as f32 / display.config.height as f32;

    let shader = create_shader(device, include_str!("assets/shader.wgsl"));
    let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) = create_uniforms(
        device,
        Uniforms {
            translation: Vector2::zero().into(),
            cursor_pos: Vector2::zero().into(),
            scale: 4.0,
            aspect_ratio,
            _padding: [0.0; 2]
        },
        0,
    );

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = create_render_pipeline(
        display,
        pipeline_layout,
        shader,
        &[VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexPositionUv>() as BufferAddress,
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
        }],
    );

    let quad_uvs = quad_uvs((0.0, 0.0), (1.0, 1.0));

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&quad_no_normal(
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            quad_uvs,
        )),
        usage: BufferUsages::VERTEX,
    });

    MandelbrotState {
        render_pipeline,
        vertex_buffer,
        uniform_buffer,
        uniform_bind_group,
        uniforms: Uniforms {
            translation: Vector2::zero().into(),
            cursor_pos: Vector2::zero().into(),
            scale: 4.0,
            aspect_ratio,
            _padding: [0.0; 2],
        },
        scale_speed: 1.0 - 0.001,
        last_cursor_location: PhysicalPosition::new(0.0, 0.0),
        cursor_location: PhysicalPosition::new(0.0, 0.0),
        last_timestep: now_secs(),
        interaction_state: InteractionState::Idle {
            last_pressed_down_at: 0.0,
            pre_tap: false,
        },
    }
}


fn render(display: &mut Display, state: &mut MandelbrotState) {
    let mut dt = now_secs() - state.last_timestep;
    #[cfg(target_arch = "wasm32")] {
        dt *= -1f32;
    }
    
    state.last_timestep = now_secs();
    state.uniforms.aspect_ratio = display.config.width as f32 / display.config.height as f32;

    

    let clear_color = Color {
        r: 100.0 / 255.0,
        g: 149.0 / 255.0,
        b: 237.0 / 255.0,
        a: 255.0 / 255.0,
    };

    state.interaction_state = match state.interaction_state.clone() {
        InteractionState::PanningIdle { pressed_down_at, pre_tap } => {
            log::info!("{} {}", now_secs(), pressed_down_at);
            if now_secs() - pressed_down_at > 0.3f32 {
                log::info!("zooming {}", now_secs());
                InteractionState::Zooming
            } else {
                InteractionState::PanningIdle { pressed_down_at, pre_tap } 
            }
        },
        state => state
    };

    match state.interaction_state {
        InteractionState::Zooming => {
            state.uniforms.scale *= state.scale_speed.powf(1f32 - dt as f32);
        },
        InteractionState::ZoomingOut => {

            log::info!("zooming out {}", state.scale_speed.powf(1f32 - dt as f32));
            state.uniforms.scale /= state.scale_speed.powf(1f32 - dt as f32);},
        _ => ()
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
    rpass.draw(0..6, 0..1);
}

fn event<'a>(display: &mut Display, state: &mut MandelbrotState, event: WindowEvent) {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            state.cursor_location = PhysicalPosition { x: position.x as f32, y: position.y as f32 };
            
            state.interaction_state = match state.interaction_state.clone() {
                InteractionState::PanningIdle { pressed_down_at: _, pre_tap: _ } | InteractionState::Zooming | InteractionState::Panning => {
                    let dx = (state.cursor_location.x - state.last_cursor_location.x) / display.size.width as f32;
                    let dy = (state.cursor_location.y - state.last_cursor_location.y) / display.size.height as f32;
                    state.uniforms.translation[0] -= dx * state.uniforms.scale;
                    state.uniforms.translation[1] += dy * state.uniforms.scale;

                    InteractionState::Panning
                },
                state => state
            };

            state.last_cursor_location = state.cursor_location;
        }
        WindowEvent::MouseInput { state: elem_state, button, .. } => {
            state.interaction_state = if elem_state == ElementState::Pressed && button == MouseButton::Left {
                state.last_cursor_location = state.cursor_location;
                match state.interaction_state.clone() {
                    InteractionState::Idle { last_pressed_down_at, pre_tap } => {
                        if pre_tap && (now_secs() - last_pressed_down_at) < 0.3f32 {
                            InteractionState::ZoomingOut
                        } else {
                            InteractionState::PanningIdle { pressed_down_at: now_secs(), pre_tap: false }
                        }
                    }
                    state => state
                }
            } else if elem_state == ElementState::Released && button == MouseButton::Left {
                match state.interaction_state.clone() {
                    InteractionState::PanningIdle { pre_tap: false, pressed_down_at } => {
                        if now_secs() - pressed_down_at < 0.3f32 {  
                            InteractionState::Idle { last_pressed_down_at: pressed_down_at, pre_tap: true }
                        } else {
                            InteractionState::Idle { last_pressed_down_at: pressed_down_at, pre_tap: false }
                        }
                    }
                    InteractionState::Zooming | InteractionState::ZoomingOut | InteractionState::Panning =>
                        InteractionState::Idle { last_pressed_down_at: now_secs(), pre_tap: false },
                    state => state
                }
            } else { state.interaction_state.clone() };

            log::info!("{}", state.interaction_state.clone());
        }
        _ => {}
    }
}
