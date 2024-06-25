use image::ImageFormat;
use image::GenericImageView;
use wgpu::util::DeviceExt;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayoutEntry;
use wgpu::Buffer;
use wgpu::Sampler;
use wgpu::{BindGroup, BindGroupLayout, Color, CommandEncoder, Device, PipelineLayout, Queue, RenderPass, RenderPipeline, ShaderModule, Surface, SurfaceTexture, TextureView, VertexBufferLayout};

use crate::display::Display;

pub mod models;

pub fn get_framebuffer(surface: &Surface) -> (SurfaceTexture, TextureView) {
    let output = surface.get_current_texture().unwrap();
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    (output, view)
}

pub fn clear_screen<'a>(
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

pub fn create_shader(device: &Device, shader_source: &'static str) -> ShaderModule {
    use std::borrow::Cow;

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
    })
}

pub fn create_render_pipeline<'a>(display: &Display, pipeline_layout: PipelineLayout, shader: ShaderModule, buffers: &[VertexBufferLayout<'a>]) -> RenderPipeline {
    display.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers,
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
    })
}


pub fn create_texture_layout_entry(device: &Device, queue: &Queue, image_bytes: &'static [u8], binding_index: u32) -> (BindGroupLayoutEntry, TextureView) {
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

    let entry = wgpu::BindGroupLayoutEntry {
        binding: binding_index,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
            
    (entry, texture_view)
}

pub fn create_sampler_entry(device: &Device, binding_index: u32) -> (BindGroupLayoutEntry, Sampler) {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    (wgpu::BindGroupLayoutEntry {
        binding: binding_index,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
        count: None,
    }, sampler)
}

pub fn create_uniform_entry(binding_index: u32) -> BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: binding_index,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn create_texture(device: &Device, layout_entries: &[BindGroupLayoutEntry], entries: &[BindGroupEntry]) -> (BindGroupLayout, BindGroup) {
    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: layout_entries
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
        label: Some("Texture Bind Group"), 
        layout: &texture_bind_group_layout, 
        entries
    });

    (texture_bind_group_layout, texture_bind_group)
}

pub fn create_uniforms<M>(device: &Device, uniform_model: M, binding_index: u32) -> (Buffer, BindGroupLayout, BindGroup) where M: bytemuck::Pod {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniform_model]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[create_uniform_entry(binding_index)],
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: binding_index,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    (uniform_buffer, uniform_bind_group_layout, uniform_bind_group)
}