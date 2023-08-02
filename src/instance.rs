use bytemuck::{Pod, Zeroable};
use cint::EncodedSrgb;
use mint::Vector2;

/// Specification of how the sprite should appear on the screen.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpriteInstance {
    /// The world position of the sprite's center.
    pub position: Vector2<f32>,
    /// Rotation (in degrees) around the sprite's center.
    pub rotation_deg: f32,
    /// RGB components of the sprite's colour.
    pub color: EncodedSrgb<u8>,
    /// The alpha channel of the drawn colour. 
    /// Any values oustide of the `0..1` range will be clamped.
    /// All the sprites are considered non-premultiplied.
    pub opacity: f32,
}

#[derive(Debug, PartialEq, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub(crate) struct RawSpriteInstance {
    pub(crate) position: [f32; 2],
    pub(crate) rotation_rad: f32,
    pub(crate) color: [f32; 4],
}

impl From<SpriteInstance> for RawSpriteInstance {
    fn from(val: SpriteInstance) -> Self {
        val.raw()
    }
}

impl SpriteInstance {
    pub(crate) fn raw(&self) -> RawSpriteInstance {
        let r = self.color.r;
        let g = self.color.g;
        let b = self.color.b;

        let [r, g, b]: [f32; 3] = [r, g, b].map(|component| (component as f32) / 255.);

        RawSpriteInstance {
            position: self.position.into(),
            rotation_rad: f32::to_radians(self.rotation_deg),
            color: [r, g, b, self.opacity].map(|component| component.clamp(0., 1.)),
        }
    }

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawSpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as _,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as _,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
