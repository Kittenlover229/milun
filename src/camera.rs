use bytemuck::{Pod, Zeroable};
use mint::{ColumnMatrix4, Vector2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vector2<f32>,
    pub size: f32,
    pub aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0., 0.].into(),
            size: 1.,
            aspect_ratio: 1.,
        }
    }
}

impl Camera {
    pub(crate) fn raw(&self) -> CameraRaw {
        CameraRaw {
            view_matrix: self.view().into(),
        }
    }

    pub(crate) fn view(&self) -> ColumnMatrix4<f32> {
        ColumnMatrix4 {
            x: [1. / (self.aspect_ratio * self.size), 0., 0., 0.].into(),
            y: [0., 1. / self.size, 0., 0.].into(),
            z: [0., 0., 1. / self.size, 0.].into(),
            w: [self.position.x, -self.position.y, 0., 1.].into(),
        }
    }
}

#[derive(Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub(crate) struct CameraRaw {
    view_matrix: [[f32; 4]; 4],
}
