use hashbrown::HashMap;
use wgpu::{util::DeviceExt, BufferUsages};

use crate::Renderer;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sprite {
    pub texture_idx: u32,
    pub indices: (u32, u32),
}

#[derive(Debug)]
pub struct Atlas {
    pub textures: (wgpu::Texture, wgpu::TextureView),
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub(crate) sprites: Vec<Sprite>,
    pub(crate) named_sprites: HashMap<String, usize>,
}

impl Atlas {
    pub fn from_device(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&Self::sampling_options());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Vertex Buffer"),
            contents: &[],
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Index Buffer"),
            contents: &[],
            usage: BufferUsages::INDEX,
        });

        let layout = &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
            label: Some("Atlas Bind Layout"),
        });

        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
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
            label: Some("Atlas Bind Group"),
        });

        Self {
            textures: (texture, texture_view),
            sampler,
            vertex_buffer,
            index_buffer,
            bind_group,
            sprites: vec![],
            named_sprites: Default::default(),
        }
    }
}

impl Atlas {
    pub fn sampling_options() -> wgpu::SamplerDescriptor<'static> {
        wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    }
}
