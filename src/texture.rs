
use graphics::{
    ImageSize
};

/// Represents a texture.
pub struct Texture {
    width: u32,
    height: u32
}

impl ImageSize for Texture {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
