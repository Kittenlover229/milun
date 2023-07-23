use image::RgbaImage;

use crate::Renderer;

#[must_use]
pub struct Atlas<'renderer> {
    pub(crate) rgba: Vec<RgbaImage>,
    pub(crate) renderer: &'renderer mut Renderer,
}

impl Atlas<'_> {
    pub fn finalize(mut self) {
        self.rgba
            .sort_by_key(|rgba| rgba.dimensions().0 * rgba.dimensions().1);
        self.rgba.reverse();

        let width = self.rgba.iter().map(|buffer| buffer.dimensions().0).sum();
        let height = self
            .rgba
            .iter()
            .map(|buffer| buffer.dimensions().1)
            .max()
            .unwrap_or(0);

        let mut buffer = RgbaImage::new(width, height);
        for pixel in buffer.pixels_mut() {
            *pixel = image::Rgba([0; 4]);
        }

        let mut i = 0;
        for buf in self.rgba {
            for (from, to) in buf.rows().zip(buffer.rows_mut()) {
                for (new_color, pixel) in from.zip(to.skip(i)) {
                    *pixel = *new_color;
                }
            }

            i += buf.dimensions().1 as usize;
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

        let texture_bind_group_layout =
            self.renderer
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
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
                label: Some("diffuse_bind_group"),
            });

        self.renderer.atlas_texture = (texture, texture_view);
        self.renderer.atlas_bind_group = bind_group;
    }

    pub fn finalize_and_repack(self) {
        todo!()
    }

    pub fn add_rgba8(mut self, buffer: RgbaImage) -> Self {
        self.rgba.push(RgbaImage::from(buffer.into()));
        self
    }
}
