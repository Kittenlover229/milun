use std::alloc::{Allocator, Global};

use cint::EncodedSrgb;
use mint::{Vector2, Vector3};

use crate::{LayerIdentifier, SpriteIndex, SpriteInstance, ViewportProperties};

pub struct SpriteRenderData {
    pub index: SpriteIndex,
    pub layer: Option<LayerIdentifier>,
    pub instance: SpriteInstance,
}

pub struct FrameData<A: Allocator> {
    pub sprite_data: Vec<SpriteRenderData, A>,
    flushed_index: usize,
}

pub struct FrameBuilder<A: Allocator = Global> {
    render_data: Vec<SpriteRenderData, A>,
    viewport: ViewportProperties,
}

impl FrameBuilder<Global> {
    pub fn new_global(viewport: ViewportProperties) -> Self {
        Self {
            render_data: vec![],
            viewport,
        }
    }
}

impl<A: Allocator> FrameBuilder<A> {
    pub fn new(viewport: ViewportProperties, allocator: A) -> Self {
        Self {
            render_data: Vec::new_in(allocator),
            viewport,
        }
    }

    pub fn viewport(&self) -> ViewportProperties {
        self.viewport
    }

    pub fn submit_sprite(
        &mut self,
        index: SpriteIndex,
        layer: Option<LayerIdentifier>,
        instance: SpriteInstance,
    ) {
        self.render_data.push(SpriteRenderData {
            index,
            layer,
            instance,
        })
    }

    pub fn finalize(self) -> FrameData<A> {
        FrameData {
            sprite_data: self.render_data,
            flushed_index: 0,
        }
    }

    pub fn draw_sprite<'me>(&'me mut self, sprite_idx: SpriteIndex) -> DrawSprite<'me, A> {
        DrawSprite {
            builder: self,
            sprite_idx,
            layer: None,
            sprite_instance: Default::default(),
        }
    }
}

#[must_use]
pub struct DrawSprite<'builder, A: Allocator> {
    builder: &'builder mut FrameBuilder<A>,
    sprite_idx: SpriteIndex,
    layer: Option<LayerIdentifier>,
    sprite_instance: SpriteInstance,
}

impl<'builder, A: Allocator> DrawSprite<'builder, A> {
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

    pub fn done(self) -> &'builder mut FrameBuilder<A> {
        self.builder
            .submit_sprite(self.sprite_idx, self.layer, self.sprite_instance);
        self.builder
    }
}
