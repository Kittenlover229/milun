use bytemuck::{Pod, Zeroable};
use mint::Vector2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vector2<f32>,
    pub zoom_factor: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0., 0.].into(),
            zoom_factor: 1.,
        }
    }
}

impl Camera {
    pub(crate) fn raw(&self) -> CameraRaw {
        CameraRaw {
            position: self.position.into(),
        }
    }
}

#[derive(Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub(crate) struct CameraRaw {
    position: [f32; 2],
}
