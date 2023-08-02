use image::{DynamicImage, RgbaImage};
use wgpu::util::DeviceExt;

use crate::{vertex::Vertex, Renderer, SpriteDrawData, SpriteIndex, SpriteLoadOptions};

/// Atlas builder used for stitching sprites together.
#[must_use]
pub struct AtlasBuilder<'renderer> {
    pub(crate) rgba: Vec<RgbaImage>,
    pub(crate) renderer: &'renderer mut Renderer,
}

impl AtlasBuilder<'_> {
    /// Stitch the sprites without packing and upload them to the GPU.
    pub fn finalize(self) -> Vec<SpriteIndex> {
        let mut sprite_indices = vec![];
        let mut sprites = vec![];
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut rgba = self.rgba;

        rgba.sort_by_key(|rgba| rgba.dimensions().0 * rgba.dimensions().1);
        rgba.reverse();

        let width = rgba.iter().map(|buffer| buffer.dimensions().0).sum();
        let height = rgba
            .iter()
            .map(|buffer| buffer.dimensions().1)
            .max()
            .unwrap_or(0);

        let mut buffer = RgbaImage::new(width, height);
        for pixel in buffer.pixels_mut() {
            *pixel = image::Rgba([0; 4]);
        }

        let mut offset = 0;
        let mut x = 0.0;
        let x_step = 1. / width as f32;
        let y_step = 1. / height as f32;

        for (i, buf) in rgba.into_iter().enumerate() {
            for (from, to) in buf.rows().zip(buffer.rows_mut()) {
                for (new_color, pixel) in from.zip(to.skip(offset)) {
                    *pixel = *new_color;
                }
            }

            let verts = [
                Vertex {
                    position: [-0.5, -0.5, 0.0],
                    tex_coords: [x, y_step * buf.height() as f32],
                },
                Vertex {
                    position: [0.5, -0.5, 0.0],
                    tex_coords: [
                        x + x_step * buf.width() as f32,
                        y_step * buf.height() as f32,
                    ],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.0],
                    tex_coords: [x, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.0],
                    tex_coords: [x + x_step * buf.width() as f32, 0.0],
                },
            ];

            let inds = [0, 1, 2, 1, 3, 2].map(|i| i + vertices.len() as u32);

            sprites.push(SpriteDrawData {
                sprite_index_range: (indices.len() as u32, (indices.len() + inds.len()) as u32),
            });

            indices.extend(inds);
            vertices.extend(verts);
            sprite_indices.push(i as SpriteIndex);
            offset += buf.dimensions().0 as usize;
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
                label: None,
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

        let texture_bind_group_layout = &self.renderer.atlas_bind_group_layout;

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
                layout: texture_bind_group_layout,
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
        self.renderer.cold_bind_group = bind_group;
        self.renderer.sprites = sprites;

        sprite_indices
    }

    pub fn finalize_and_repack(self) {
        todo!()
    }

    /// Add a sprite into the queue later to be stitched (and maybe packed).
    pub fn add_sprite(self, image: impl Into<DynamicImage>) -> Self {
        self.add_sprite_advanced(
            image,
            SpriteLoadOptions {
                ..Default::default()
            },
        )
    }

    /// Add a sprite with options not activated by default
    pub fn add_sprite_advanced(
        mut self,
        image: impl Into<DynamicImage>,
        options: impl Into<SpriteLoadOptions>,
    ) -> Self {
        let SpriteLoadOptions { premultiplied, .. } = options.into();
        let mut rgba32f = DynamicImage::from(image.into()).to_rgb32f();

        if premultiplied {
            for [r, g, b, a] in rgba32f.iter_mut().array_chunks::<4>() {
                *r /= *a;
                *g /= *a;
                *b /= *a;
            }
        }

        self.rgba.push(DynamicImage::from(rgba32f).to_rgba8());
        self
    }
}
