use bytemuck::{Pod, Zeroable};
use mint::{ColumnMatrix4, Vector2};

/// An orthographic camera. Projecting into the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Current position of the camera relative to which everything is rendered.
    pub position: Vector2<f32>,
    /// Scale of the camera's viewport
    pub size: f32,
    /// Aspect ratio of the viewport everything is rendered onto
    pub aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0., 0.].into(),
            size: 3.,
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

    /// Calculate the view matrix
    pub fn view(&self) -> ColumnMatrix4<f32> {
        ColumnMatrix4 {
            x: [1. / (self.aspect_ratio * self.size), 0., 0., 0.].into(),
            y: [0., 1. / self.size, 0., 0.].into(),
            z: [0., 0., 1. / self.size, 0.].into(),
            w: [
                self.position.x / self.size,
                -self.position.y / self.size,
                0.,
                1.,
            ]
            .into(),
        }
    }
}

#[derive(Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub(crate) struct CameraRaw {
    view_matrix: [[f32; 4]; 4],
}
