use crate::{
    packing::Node, vertex::Vertex, Renderer, SpriteDrawData, SpriteIndex, SpriteLoadOptions,
};
use image::{DynamicImage, RgbaImage};
use mint::Vector2;
use wgpu::util::DeviceExt;

/// Atlas builder used for stitching sprites together.
#[must_use]
pub struct AtlasBuilder<'renderer, const N: usize> {
    pub(crate) rgba: Vec<RgbaImage>,
    pub(crate) renderer: &'renderer mut Renderer,
    pub(crate) sprites: [SpriteIndex; N],
}

impl<'me, const N: usize> AtlasBuilder<'me, N> {
    #[must_use]
    pub fn finalize_and_repack(mut self) -> [SpriteIndex; N] {
        self.rgba.sort_by(|lhs, rhs| {
            match lhs.width().cmp(&rhs.width()) {
                std::cmp::Ordering::Equal => lhs.height().cmp(&rhs.height()),
                x => x,
            }
            .reverse()
        });

        let mut root = Node {
            children: None,
            topleft: [0; 2].into(),
            botright: [u32::MAX; 2].into(),
            image_index: None,
        };

        for (i, image) in self.rgba.iter().enumerate() {
            root.insert(image, i);
        }

        let mut node_queue = vec![root];
        let mut sprites_to_put = vec![];
        while let Some(node) = node_queue.pop() {
            let Node {
                children,
                topleft,
                image_index,
                ..
            } = node;

            if let Some([left, right]) = children {
                node_queue.push(left.into_inner());
                node_queue.push(right.into_inner());
            }

            if let Some(idx) = image_index {
                sprites_to_put.push((topleft, idx))
            }
        }

        let buffer_size: Vector2<u32> = sprites_to_put
            .iter()
            .map(|(pos, idx)| {
                let z = &self.rgba[*idx];
                [pos.x + z.width(), pos.y + z.height()]
            })
            .reduce(|[x1, y1], [x2, y2]| [x1.max(x2), y1.max(y2)])
            .unwrap()
            .into();

        let (width, height) = (buffer_size.x, buffer_size.y);

        let mut buffer = RgbaImage::new(buffer_size.x, buffer_size.y);
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut sprites = vec![];

        for (pos, idx) in sprites_to_put.iter() {
            let sprite = &mut self.rgba[*idx];
            for (row, new_row) in buffer.rows_mut().skip(pos.y as _).zip(sprite.rows_mut()) {
                for (pixel, new_color) in row.skip(pos.x as _).zip(new_row) {
                    *pixel = *new_color;
                }
            }

            let x_step = 1. / width as f32;
            let y_step = 1. / height as f32;
            let x = pos.x as f32 / width as f32;
            let y = pos.y as f32 / height as f32;

            let verts = [
                Vertex {
                    position: [-0.5, -0.5, 0.0],
                    tex_coords: [x, y + y_step * sprite.height() as f32],
                },
                Vertex {
                    position: [0.5, -0.5, 0.0],
                    tex_coords: [
                        x + x_step * sprite.width() as f32,
                        y + y_step * sprite.height() as f32,
                    ],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.0],
                    tex_coords: [x, y],
                },
                Vertex {
                    position: [0.5, 0.5, 0.0],
                    tex_coords: [x + x_step * sprite.width() as f32, y],
                },
            ];

            let inds = [0, 1, 2, 1, 3, 2].map(|i| i + vertices.len() as u32);

            sprites.push(SpriteDrawData {
                sprite_index_range: (indices.len() as u32, (indices.len() + inds.len()) as u32),
            });
            indices.extend(inds);
            vertices.extend(verts);
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
                    label: None,
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        self.renderer.vertex_buffer =
            self.renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
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

        self.sprites.reverse();
        self.sprites
    }

    /// Add a sprite into the queue later to be stitched (and maybe packed).
    pub fn add_sprite(self, image: impl Into<DynamicImage>) -> AtlasBuilder<'me, { N + 1 }> {
        self.add_sprite_advanced(
            image,
            SpriteLoadOptions {
                ..Default::default()
            },
        )
    }

    /// Add a sprite with options not activated by default
    pub fn add_sprite_advanced(
        self,
        image: impl Into<DynamicImage>,
        options: impl Into<SpriteLoadOptions>,
    ) -> AtlasBuilder<'me, { N + 1 }> {
        let AtlasBuilder {
            mut rgba,
            renderer,
            sprites,
        } = self;

        let SpriteLoadOptions { premultiplied, .. } = options.into();
        let mut rgba32f = image.into().to_rgb32f();

        if premultiplied {
            for [r, g, b, a] in rgba32f.iter_mut().array_chunks::<4>() {
                *r /= *a;
                *g /= *a;
                *b /= *a;
            }
        }

        let len = renderer.sprites.len();
        let mut sprites_iterator = sprites
            .into_iter()
            .chain(std::iter::once(sprites.len() + len));
        rgba.push(DynamicImage::from(rgba32f).to_rgba8());

        AtlasBuilder {
            rgba,
            renderer,
            sprites: [(); N + 1].map(|_| sprites_iterator.next().unwrap()),
        }
    }
}
