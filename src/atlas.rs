use std::ops::Range;

use image::{DynamicImage, RgbaImage};
use wgpu::util::DeviceExt;

use crate::{vertex::Vertex, Renderer};

pub type SpriteIndex = usize;

pub struct Sprite {
    pub sprite_index_range: (u32, u32),
}

impl Sprite {
    pub fn indices(&self) -> Range<u32> {
        self.sprite_index_range.0 as u32..self.sprite_index_range.1 as u32
    }
}

#[must_use]
pub struct Atlas<'renderer> {
    pub(crate) rgba: Vec<RgbaImage>,
    pub(crate) renderer: &'renderer mut Renderer,
}

impl Atlas<'_> {
    pub fn finalize(mut self) -> Vec<SpriteIndex> {
        let mut sprite_indices = vec![];
        let mut sprites = vec![];
        let mut vertices = vec![];
        let mut indices = vec![];

        self.rgba
            .sort_by_key(|rgba| rgba.dimensions().0 * rgba.dimensions().1);
        self.rgba.reverse();

        let width = self.rgba.iter().map(|buffer| buffer.dimensions().0).sum();
        let height = self
            .rgba
            .iter()
            .map(|buffer| buffer.dimensions().1)
            .fold(0, |acc, x| acc.max(x));

        let mut buffer = RgbaImage::new(width, height);
        for pixel in buffer.pixels_mut() {
            *pixel = image::Rgba([0; 4]);
        }

        let mut offset = 0;
        let mut x = 0.0;
        let x_step = 1. / width as f32;
        let y_step = 1. / height as f32;

        for (i, buf) in self.rgba.into_iter().enumerate() {
            for (from, to) in buf.rows().zip(buffer.rows_mut()) {
                for (new_color, pixel) in from.zip(to.skip(offset)) {
                    *pixel = *new_color;
                }
            }

            let verts = [
                Vertex {
                    position: [-0.5, -0.5, 0.0],
                    tex_coords: [0.0, y_step * buf.height() as f32],
                },
                Vertex {
                    position: [0.5, -0.5, 0.0],
                    tex_coords: [x_step * buf.width() as f32, y_step * buf.height() as f32],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.0],
                    tex_coords: [x_step * buf.width() as f32, 0.0],
                },
            ]
            .map(|mut v| {
                v.tex_coords[0] += x;
                v
            });

            let inds = [0, 1, 2, 1, 3, 2]
                .into_iter()
                .map(|i| i + vertices.len() as u32);

            sprites.push(Sprite {
                sprite_index_range: (indices.len() as u32, (indices.len() + inds.len()) as u32),
            });

            indices.extend(inds);
            vertices.extend(verts);
            sprite_indices.push(i as SpriteIndex);
            offset += buf.dimensions().1 as usize;
            x += buf.width() as f32 * x_step;
        }

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self
            .renderer
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("texture"),
                view_formats: &[],
            });

        self.renderer.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_bind_group_layout = &self.renderer.layout;

        self.renderer.index_buffer =
            self.renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Atlas Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        self.renderer.vertex_buffer =
            self.renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Atlas Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let bind_group = self
            .renderer
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.renderer.atlas_sampler),
                    },
                ],
                label: None,
            });

        self.renderer.atlas_texture = (texture, texture_view);
        self.renderer.atlas_bind_group = bind_group;
        self.renderer.sprites = sprites;

        sprite_indices
    }

    pub fn finalize_and_repack(self) {
        todo!()
    }

    pub fn add_image(mut self, buffer: impl Into<DynamicImage>) -> Self {
        self.rgba.push(DynamicImage::from(buffer.into()).to_rgba8());
        self
    }
}
