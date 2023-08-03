use std::cell::RefCell;

use image::{DynamicImage, RgbaImage};
use mint::Vector2;
use wgpu::util::DeviceExt;

use crate::{vertex::Vertex, Renderer, SpriteDrawData, SpriteIndex, SpriteLoadOptions};

/// Atlas builder used for stitching sprites together.
#[must_use]
pub struct AtlasBuilder<'renderer, const N: usize> {
    pub(crate) rgba: Vec<RgbaImage>,
    pub(crate) renderer: &'renderer mut Renderer,
    pub(crate) sprites: [SpriteIndex; N],
}

impl<'me, const N: usize> AtlasBuilder<'me, N> {
    /// Stitch the sprites without packing and upload them to the GPU.
    #[must_use]
    pub fn finalize(self) -> [SpriteIndex; N] {
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

        self.sprites
    }

    #[must_use]
    pub fn finalize_and_repack(mut self) -> [SpriteIndex; 3] {
        #[derive(Debug)]
        struct Node {
            children: Option<(Box<RefCell<Node>>, Box<RefCell<Node>>)>,
            topleft: Vector2<u32>,
            botright: Vector2<u32>,
            image_index: Option<usize>,
        }

        self.rgba.sort_by(|lhs, rhs| {
            match lhs.width().cmp(&rhs.width()) {
                std::cmp::Ordering::Equal => lhs.height().cmp(&rhs.height()),
                x => x,
            }
            .reverse()
        });

        impl Node {
            // Returns a new Some(Some(node)) if a new node should be created, a None if no node
            // can be created, or Some(None) if the node fit inside itself
            pub fn insert(&mut self, image: &RgbaImage, index: usize) -> bool {
                match &self.children {
                    Some((left, right)) => {
                        if left.borrow_mut().insert(image, index) {
                            true
                        } else {
                            right.borrow_mut().insert(image, index)
                        }
                    }

                    None => {
                        if self.image_index != None {
                            return false;
                        }

                        let img_size: Vector2<u32> = [image.width(), image.height()].into();

                        if self.botright.x < img_size.x || self.botright.y < img_size.y {
                            return false;
                        }

                        if img_size == self.botright {
                            self.image_index = Some(index);
                            return true;
                        }

                        let dx = (self.botright.x - self.topleft.x) - img_size.x;
                        let dy = (self.botright.y - self.topleft.y) - img_size.y;

                        let children = if dx > dy {
                            [
                                Node {
                                    children: None,
                                    topleft: self.topleft,
                                    botright: [self.topleft.x + img_size.x - 1, self.botright.y]
                                        .into(),
                                    image_index: Some(index),
                                },
                                Node {
                                    children: None,
                                    topleft: [self.topleft.x + img_size.x, self.topleft.y].into(),
                                    botright: self.botright,
                                    image_index: None,
                                },
                            ]
                        } else {
                            [
                                Node {
                                    children: None,
                                    topleft: self.topleft,
                                    botright: [self.botright.x, self.topleft.y + img_size.y - 1]
                                        .into(),
                                    image_index: Some(index),
                                },
                                Node {
                                    children: None,
                                    topleft: [self.topleft.x, self.topleft.y + img_size.y].into(),
                                    botright: self.botright,
                                    image_index: None,
                                },
                            ]
                        };

                        let [a, b] = children;
                        self.children =
                            Some((Box::new(RefCell::new(a)), Box::new(RefCell::new(b))));
                        true
                    }
                }
            }
        }

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

            if let Some((left, right)) = children {
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

        let mut buffer = RgbaImage::new(buffer_size.x, buffer_size.y);
        for (pos, idx) in sprites_to_put.iter() {
            let sprite = &mut self.rgba[*idx];
            for (row, new_row) in buffer.rows_mut().skip(pos.y as _).zip(sprite.rows_mut()) {
                for (pixel, new_color) in row.skip(pos.x as _).zip(new_row) {
                    *pixel = *new_color;
                }
            }
        }

        buffer.save("E.png").unwrap();
        [0, 0, 0]
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
