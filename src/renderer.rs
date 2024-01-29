use std::cell::OnceCell;

use chrono::{Duration, NaiveDateTime, Utc};
use cint::EncodedSrgb;

#[cfg(feature = "egui")]
use crate::egui::EguiIntegration;
#[cfg(feature = "egui")]
use egui::Context;

use mint::{Vector2, Vector3};
use wgpu::{util::DeviceExt, Buffer, BufferDescriptor, BufferUsages};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{
    vertex::Vertex, AtlasBuilder, Camera, CameraRaw, LayerIdentifier, RawSpriteInstance,
    SpriteDrawData, SpriteIndex, SpriteInstance,
};

/// The main object used for drawing. Use `.atlas()` to load new sprites and
/// `.begin_frame()` to draw them.
pub struct Renderer {
    /*** WGPU Data ***/
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) render_pipeline: wgpu::RenderPipeline,

    /*** Window Data ***/
    /// Actual size of the window in pixels (?)
    pub(crate) size: PhysicalSize<u32>,
    /// Window everything is presented onto
    pub(crate) window: Window,

    /*** Atlas ***/
    /// A stitched texture generate by the AtlasBuilder
    pub(crate) atlas_texture: (wgpu::Texture, wgpu::TextureView),
    /// The layout of the atlas bind group
    pub(crate) atlas_bind_group_layout: wgpu::BindGroupLayout,
    /// Sampler used by the atlas
    pub(crate) atlas_sampler: wgpu::Sampler,

    /*** Camera ***/
    /// The camera from the perspective of which everything is rendered
    pub(crate) camera: Camera,
    /// Camera's reflection on the GPU
    pub(crate) camera_buffer: wgpu::Buffer,
    /// The background color to clear the buffer with
    pub(crate) clear_color: Option<EncodedSrgb<u8>>,

    /*** Misc ***/
    /// To avoid using magic numbers you can name layers. The layer names are
    /// resolved from this table.
    pub(crate) named_layers: hashbrown::HashMap<String, i32>,

    /*** Bind Groups ***/
    /// Bind Group for data that rarely changes
    pub(crate) cold_bind_group: wgpu::BindGroup,
    /// Bind Group for data that changes often
    pub(crate) hot_bind_group: wgpu::BindGroup,

    /*** Instances ***/
    /// The amount of instances the `instance_buffer` can take
    pub(crate) instance_count: u64,
    /// The buffer that all data is in
    pub(crate) instance_buffer: Buffer,

    /*** Sprites ***/
    /// The indices into the index buffer for each sprite
    pub(crate) sprites: Vec<SpriteDrawData>,
    /// Vertices of the sprite's mesh
    pub(crate) vertex_buffer: Buffer,
    /// Indices into the vertex buffer
    pub(crate) index_buffer: Buffer,

    /*** Timers ***/
    /// The timestamp of the first rendered frame
    pub(crate) first_render_ts: OnceCell<NaiveDateTime>,
    pub(crate) last_render_ts: Option<NaiveDateTime>,
    pub(crate) delta_time: Duration,

    /*** EGUI Integration ***/
    #[cfg(feature = "egui")]
    pub egui_integration: EguiIntegration,
}

#[cfg(feature = "sync-new")]
impl From<Window> for Renderer {
    fn from(window: Window) -> Self {
        pollster::block_on(Renderer::new(window))
    }
}

impl Renderer {
    /// Creates all the required objects on the GPU to draw onto the Window
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
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/main.wgsl").into()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
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
                label: None,
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
                label: None,
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&cold_bind_group_layout, &hot_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            label: None,
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
            label: None,
        });

        let camera = Camera::default();

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
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
            label: None,
        });

        Self {
            clear_color: None,
            cold_bind_group,
            hot_bind_group,
            atlas_bind_group_layout: cold_bind_group_layout,

            #[cfg(feature = "egui")]
            egui_integration: { EguiIntegration::new(&window, &device, surface_format) },

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
            named_layers: [].into(),

            first_render_ts: OnceCell::new(),
            last_render_ts: None,
            delta_time: Duration::zero(),
        }
    }
}

impl Renderer {
    /// Handle window inputs that influence the renderer, returns `true` if
    /// the input was consumed and `false` otherwise.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        #[cfg(feature = "egui")]
        if self.egui_integration.input(event) {
            return true;
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
                true
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
                true
            }
            _ => false,
        }
    }

    /// Creates a temporary atlas builder that is used to insert new sprite information.
    pub fn atlas(&mut self) -> AtlasBuilder<'_, 0> {
        AtlasBuilder {
            renderer: self,
            rgba: vec![],
            statically_dispatched_sprites: [],
            dynamically_dispatched_sprites: vec![],
        }
    }

    fn sampling_options() -> wgpu::SamplerDescriptor<'static> {
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

    /// Changes the size of the output surface and refreshes the camera.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.mutate_camera(|camera| {
                camera.aspect_ratio = new_size.width as f32 / new_size.height as f32
            });
        }
    }

    /// Start the rendering process by creating a frame buffer.
    pub fn begin_frame(&mut self) -> FrameBuilder<'_> {
        FrameBuilder::from(self)
    }

    /// Change the camera data and automatically refresh the buffer
    pub fn mutate_camera(&mut self, camera: impl FnOnce(&mut Camera)) {
        camera(&mut self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera.raw()),
        );
    }

    pub fn set_layer(&mut self, name: impl ToString, constant: i32) {
        self.named_layers.insert(name.to_string(), constant);
    }

    pub fn window_to_world(&self, window_position: impl Into<Vector2<u32>>) -> Vector2<f32> {
        let [x, y] = <[u32; 2]>::from(window_position.into()).map(|v| v as f32);
        let [x, y] = [
            (x / self.window.inner_size().width as f32 * 2. - 1.)
                * self.camera.size
                * self.camera.aspect_ratio,
            (-y / self.window.inner_size().height as f32 * 2. + 1.) * self.camera.size,
        ];

        [x, y].into()
    }

    pub(crate) fn reserve_instance_buffer_for(&mut self, new_instance_count: u64) {
        if new_instance_count > self.instance_count {
            self.instance_count = self.instance_count.max(new_instance_count);
            self.instance_buffer = self.device.create_buffer(&BufferDescriptor {
                label: None,
                size: self.instance_count * std::mem::size_of::<RawSpriteInstance>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
    }

    pub(crate) fn resolve_layer_ord(&mut self, layer: Option<&LayerIdentifier>) -> i32 {
        match layer {
            None => 0,
            Some(LayerIdentifier::Ordinal(c)) => *c,
            Some(LayerIdentifier::Named(name)) => {
                *self.named_layers.get(name.as_ref()).unwrap_or(&0)
            }
        }
    }
}

impl Renderer {
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

#[must_use]
pub struct DrawSprite<'builder, 'renderer> {
    renderer: &'builder mut FrameBuilder<'renderer>,
    sprite_idx: SpriteIndex,
    layer: Option<LayerIdentifier>,
    sprite_instance: SpriteInstance,
}

impl<'builder, 'renderer> DrawSprite<'builder, 'renderer> {
    pub fn layer(self, layer: impl Into<LayerIdentifier>) -> Self {
        Self {
            layer: Some(layer.into()),
            ..self
        }
    }

    pub fn pos(mut self, pos: impl Into<Vector3<f32>>) -> Self {
        self.sprite_instance.position = pos.into();
        self
    }

    pub fn opacity(mut self, opacity: impl Into<f32>) -> Self {
        self.sprite_instance.opacity = opacity.into();
        self
    }

    pub fn color(mut self, color: impl Into<EncodedSrgb<u8>>) -> Self {
        self.sprite_instance.color = color.into();
        self
    }

    pub fn stretch(mut self, scale: impl Into<Vector2<f32>>) -> Self {
        self.sprite_instance.transform.scale = scale.into();
        self
    }

    pub fn scale(mut self, size: impl Into<f32>) -> Self {
        let size: f32 = size.into();
        self.sprite_instance.transform.scale = [size, size].into();
        self
    }

    pub fn rotate(mut self, rad: f32) -> Self {
        self.sprite_instance.transform.rotation_rad = rad;
        self
    }

    pub fn draw(self) -> &'builder mut FrameBuilder<'renderer> {
        self.renderer
            .draw_sprite(self.sprite_idx, self.layer, self.sprite_instance);
        self.renderer
    }
}

/// Used for adding draw data to a frame of a [`Renderer`].
#[must_use]
pub struct FrameBuilder<'renderer> {
    renderer: &'renderer mut Renderer,
    draw_sprites: Vec<(SpriteIndex, (Option<LayerIdentifier>, SpriteInstance))>,
}

impl<'renderer> From<&'renderer mut Renderer> for FrameBuilder<'renderer> {
    fn from(renderer: &'renderer mut Renderer) -> Self {
        #[cfg(feature = "egui")]
        renderer.egui_integration.begin_frame(&renderer.window);
        FrameBuilder {
            renderer,
            draw_sprites: vec![],
        }
    }
}

impl<'renderer> FrameBuilder<'renderer> {
    pub fn sprite(&mut self, sprite_idx: SpriteIndex) -> DrawSprite<'_, 'renderer> {
        DrawSprite {
            renderer: self,
            layer: None,
            sprite_idx,
            sprite_instance: SpriteInstance::default(),
        }
    }

    /// Draws the sprite at index `sprite_index` in the Renderer's sprite list.
    ///
    /// The sprite is displayed at `position`, tinted with colour `color` and
    /// opacity `opacity`.
    ///
    /// All the sprites are considered non-premultiplied by default.
    /// If the sprite you are drawing is premultiplied, specify that option in
    /// the [`AtlasBuilder`] using the [`SpriteLoadOptions`].
    #[deprecated = "Use `sprite` instead"]
    pub fn draw_sprite(
        &mut self,
        sprite_idx: SpriteIndex,
        layer_id: Option<impl Into<LayerIdentifier>>,
        instance: impl Into<SpriteInstance>,
    ) -> &mut Self {
        self.draw_sprites
            .push((sprite_idx, (layer_id.map(|x| x.into()), instance.into())));
        self
    }

    #[cfg(feature = "egui")]
    pub fn draw_egui(self, show: impl FnOnce(&Context)) -> Self {
        show(&self.renderer.egui_integration.context);
        self
    }

    /// Finishes the frame and presents it onto the [`Renderer`]'s surface.
    pub fn end_frame(mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.renderer.surface.get_current_texture()?;
        let sprite_to_draw_count = self.draw_sprites.len();

        let now = Utc::now().naive_utc();
        self.renderer.first_render_ts.get_or_init(|| now);
        if let Some(last_ts) = self.renderer.last_render_ts {
            self.renderer.delta_time = now - last_ts;
        }
        self.renderer.last_render_ts = Some(now);

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer
            .reserve_instance_buffer_for(sprite_to_draw_count as _);

        let mut encoder = self
            .renderer
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.draw_sprites.sort_by(
            |(sprite_idx_lhs, (layer_lhs, _)), (sprite_idx_rhs, (layer_rhs, _))| {
                let layer_lhs = self.renderer.resolve_layer_ord(layer_lhs.as_ref());
                let layer_rhs = self.renderer.resolve_layer_ord(layer_rhs.as_ref());

                match layer_lhs.cmp(&layer_rhs) {
                    std::cmp::Ordering::Equal => sprite_idx_lhs.cmp(sprite_idx_rhs),
                    x => x,
                }
            },
        );

        let Renderer {
            queue,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            index_buffer,
            sprites,
            cold_bind_group,
            hot_bind_group,
            #[cfg(feature = "egui")]
            egui_integration,
            #[cfg(feature = "egui")]
            window,
            #[cfg(feature = "egui")]
            device,
            #[cfg(feature = "egui")]
            config,
            clear_color,
            ..
        } = self.renderer;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(color) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: color.r as f64 / 255.,
                                g: color.g as f64 / 255.,
                                b: color.b as f64 / 255.,
                                a: 1.0,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(render_pipeline);

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.set_bind_group(0, cold_bind_group, &[]);
            render_pass.set_bind_group(1, hot_bind_group, &[]);

            let instances = self.draw_sprites.into_iter().fold(
                Vec::with_capacity(sprite_to_draw_count),
                |mut instances, (sprite_idx, (_, instance))| {
                    match instances.last_mut() {
                        Some((current_sprite, _)) if *current_sprite != sprite_idx => {
                            instances.push((sprite_idx, vec![instance]));
                        }
                        None => {
                            instances.push((sprite_idx, vec![instance]));
                        }
                        Some((_, instances_of_this_sprite)) => {
                            instances_of_this_sprite.push(instance);
                        }
                    };
                    instances
                },
            );

            let mut instances_processed = 0;
            for (sprite_idx, instances) in instances.iter() {
                render_pass.draw_indexed(
                    sprites[*sprite_idx].indices(),
                    0,
                    instances_processed as _..(instances_processed + instances.len()) as _,
                );

                instances_processed += instances.len();
            }

            queue.write_buffer(
                instance_buffer,
                0,
                bytemuck::cast_slice(
                    &instances
                        .into_iter()
                        .flat_map(|(_, i)| i)
                        .map(|i| i.raw())
                        .collect::<Vec<_>>(),
                ),
            );
        }

        #[cfg(feature = "egui")]
        egui_integration.end_frame(window, device, queue, config, &view, &mut encoder);

        queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn done(&mut self) {}

    pub fn renderer(&mut self) -> &mut Renderer {
        self.renderer
    }
}
