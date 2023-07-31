use std::cell::{RefCell, RefMut};

use bytemuck::{Pod, Zeroable};
use mint::Vector2;
use wgpu::{util::DeviceExt, Buffer, BufferDescriptor, BufferUsages};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{AtlasBuilder, Camera, CameraRaw, Sprite, SpriteIndex, Vertex};

pub struct Renderer {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) render_pipeline: wgpu::RenderPipeline,

    pub(crate) size: PhysicalSize<u32>,
    pub(crate) window: Window,

    pub(crate) atlas_texture: (wgpu::Texture, wgpu::TextureView),
    pub(crate) atlas_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) atlas_sampler: wgpu::Sampler,

    pub(crate) camera: RefCell<Camera>,
    pub(crate) camera_buffer: wgpu::Buffer,

    // Bind Group for data that rarely changes
    pub(crate) cold_bind_group: wgpu::BindGroup,
    // Bind Group for data that changes often
    pub(crate) hot_bind_group: wgpu::BindGroup,

    pub(crate) instance_count: u64,
    pub(crate) instance_buffer: Buffer,

    pub(crate) sprites: Vec<Sprite>,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
}

impl From<Window> for Renderer {
    fn from(window: Window) -> Self {
        pollster::block_on(Renderer::new(window))
    }
}

impl Renderer {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .find(|adapter| adapter.is_surface_supported(&surface))
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/main.wgsl").into()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<RawSpriteInstance>() as _,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&Self::sampling_options());

        let cold_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("Cold Bind Layout"),
            });

        let hot_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("Hot Bind Layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&cold_bind_group_layout, &hot_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), SpriteInstance::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas Texture"),
            view_formats: &[],
        });

        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let cold_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cold_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Cold Bind Group"),
        });

        let camera = Camera::default();

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&camera_buffer, 0, bytemuck::bytes_of(&camera.raw()));

        let hot_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &hot_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Cold Bind Group"),
        });

        Self {
            cold_bind_group,
            hot_bind_group,
            atlas_bind_group_layout: cold_bind_group_layout,

            camera: Default::default(),
            camera_buffer,

            atlas_texture: (atlas_texture, atlas_texture_view),
            instance_count: 1,
            instance_buffer,
            size,
            sprites: vec![],
            atlas_sampler: sampler,
            vertex_buffer,
            index_buffer,
            window,
            surface,
            device,
            queue,
            config,
            render_pipeline,
        }
    }
}

impl Renderer {
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
                false
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
                false
            }
            _ => false,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn camera_mut(&self) -> RefMut<'_, Camera> {
        self.camera.borrow_mut()
    }

    pub fn atlas(&mut self) -> AtlasBuilder<'_> {
        AtlasBuilder {
            renderer: self,
            rgba: vec![],
        }
    }

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

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera_mut().aspect_ratio = new_size.width as f32 / new_size.height as f32;
            self.refresh_camera();
        }
    }

    pub fn refresh_camera(&mut self) {
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera_mut().raw()),
        );
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn begin_frame(&mut self) -> FrameBuilder<'_> {
        FrameBuilder {
            renderer: self,
            draw_sprites: vec![],
        }
    }

    pub(crate) fn reserve_instance_buffer_for(&mut self, new_instance_count: u64) {
        if new_instance_count > self.instance_count {
            self.instance_count = new_instance_count;
            self.instance_buffer = self.device.create_buffer(&BufferDescriptor {
                label: None,
                size: self.instance_count * std::mem::size_of::<RawSpriteInstance>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpriteInstance {
    position: Vector2<f32>,
}

#[derive(Debug, PartialEq, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub(crate) struct RawSpriteInstance {
    position: [f32; 2],
}

impl SpriteInstance {
    pub(crate) fn raw(&self) -> RawSpriteInstance {
        RawSpriteInstance {
            position: self.position.into(),
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawSpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[must_use]
pub struct FrameBuilder<'renderer> {
    renderer: &'renderer mut Renderer,
    draw_sprites: Vec<(SpriteIndex, SpriteInstance)>,
}

impl FrameBuilder<'_> {
    pub fn draw_sprite_indexed(
        mut self,
        sprite_idx: SpriteIndex,
        position: impl Into<Vector2<f32>>,
    ) -> Self {
        self.draw_sprites.push((
            sprite_idx,
            SpriteInstance {
                position: position.into(),
            },
        ));
        self
    }

    pub fn end_frame(self) -> Result<(), wgpu::SurfaceError> {
        let output = self.renderer.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer
            .reserve_instance_buffer_for(self.draw_sprites.len() as _);

        let mut encoder =
            self.renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.renderer.render_pipeline);

            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.renderer.instance_buffer.slice(..));

            render_pass.set_index_buffer(
                self.renderer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.set_bind_group(0, &self.renderer.cold_bind_group, &[]);
            render_pass.set_bind_group(1, &self.renderer.hot_bind_group, &[]);

            let mut instances = vec![];
            let mut last_sprite_idx = 0;
            let mut same_sprite_len = 0;
            let mut same_sprite_sequence_begin_index = 0;
            for (sprite_idx, sprite_instance) in self.draw_sprites {
                if sprite_idx == last_sprite_idx {
                    same_sprite_len += 1;
                } else {
                    let sprite_mesh_indices =
                        self.renderer.sprites[last_sprite_idx as usize].indices();

                        render_pass.draw_indexed(
                        sprite_mesh_indices,
                        0,
                        0..same_sprite_len + same_sprite_len,
                    );

                    last_sprite_idx = sprite_idx;
                    same_sprite_len = 1;
                    same_sprite_sequence_begin_index = instances.len() as u32;
                }

                instances.push(sprite_instance.raw());
            }

            if !instances.is_empty() {
                render_pass.draw_indexed(
                    self.renderer.sprites[last_sprite_idx as usize].indices(),
                    0,
                    same_sprite_sequence_begin_index..instances.len() as u32,
                );
            };

            self.renderer.queue.write_buffer(
                &self.renderer.instance_buffer,
                0,
                bytemuck::cast_slice(&instances),
            );
        }

        self.renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
