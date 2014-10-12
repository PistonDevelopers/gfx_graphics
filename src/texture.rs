use gfx;
use image;
use image::{
    GenericImage,
    ImageBuf,
    Rgba,
};
use graphics::{
    ImageSize
};

/// Represents a texture.
pub struct Texture {
    /// A handle to the Gfx texture.
    pub handle: gfx::TextureHandle,
}

impl Texture {
    /// Creates a texture from path.
    pub fn from_path<
        C: gfx::CommandBuffer,
        D: gfx::Device<C>
    >(device: &mut D, path: &Path) -> Result<Texture, String> {
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
        let texture_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Texture2D,
            format: gfx::tex::RGBA8,
        };
        let image_info = texture_info.to_image_info();
        let texture = device.create_texture(texture_info).unwrap();
        device.update_texture(&texture, &image_info,
            img.raw_pixels().as_slice())
        .unwrap();

        Ok(Texture {
            handle: texture
        })
    }
    
    /// Creates a texture from image.
    pub fn from_image<
        C: gfx::CommandBuffer,
        D: gfx::Device<C>
    >(device: &mut D, image: &ImageBuf<Rgba<u8>>) -> Texture {
        let (width, height) = image.dimensions();
        let texture_info = gfx::tex::TextureInfo {
            width: width as u16,
            height: height as u16,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Texture2D,
            format: gfx::tex::RGBA8,
        };
        let image_info = texture_info.to_image_info();
        let texture = device.create_texture(texture_info).unwrap();
        device.update_texture(&texture, &image_info,
            image.pixelbuf().as_slice())
        .unwrap();

        Texture {
            handle: texture
        }
    }
}

impl ImageSize for Texture {
    fn get_size(&self) -> (u32, u32) {
        let info = self.handle.get_info();
        (info.width as u32, info.height as u32)
    }
}
