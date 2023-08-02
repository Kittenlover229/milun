use std::ops::Range;

pub type SpriteIndex = usize;

/// Options of the sprite when it's loaded into the sprite atlas and stitched.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct SpriteLoadOptions {
    /// Premultiplied images are unpremultiplied before storage.
    /// This flag specifies if it should be done to a sprite.
    pub premultiplied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SpriteDrawData {
    pub sprite_index_range: (u32, u32),
}

impl SpriteDrawData {
    pub fn indices(&self) -> Range<u32> {
        self.sprite_index_range.0..self.sprite_index_range.1
    }
}
