use image;
use image::GenericImage;
use graphics::{
    ImageSize
};

/// Represents a texture.
pub struct Texture {
    width: u32,
    height: u32
}

impl Texture {
    /// Creates a texture from path.
    pub fn from_path(path: &Path) -> Result<Texture, String> {
        let img = match image::open(path) {
            Ok(img) => img,
            Err(e)  => return Err(format!("Could not load '{}': {}",
                path.filename_str().unwrap(), e)),
        };

        match img.color() {
            image::RGBA(8) => {},
            c => fail!("Unsupported color type {} in png", c),
        };

        let (width, height) = img.dimensions();

        Ok(Texture {
            width: width,
            height: height
        })
    }
}

impl ImageSize for Texture {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
