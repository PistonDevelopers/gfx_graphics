//! Glyph caching

use std::path::Path;
use std::collections::hash_map::{ HashMap, Entry };
use { freetype, graphics };
use gfx_lib as gfx;

/// The type alias for the font size.
pub type FontSize = u32;

/// The type alias for font characters.
pub type Character<D> = graphics::character::Character<::Texture<D>>;

/// An enum to represent various possible run-time errors that may occur.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    /// An error happened when creating a gfx texture.
    Texture(gfx::tex::TextureError),
    /// An error happened with the FreeType library.
    Freetype(freetype::error::Error),
}

/// A struct used for caching rendered font.
pub struct GlyphCache<'a, R: gfx::Resources> {
    /// The font face.
    pub face: freetype::Face<'a>,
    empty_texture: ::Texture<R>,
    data: HashMap<(FontSize, char), Character<R>>,
}

impl<'a, R> GlyphCache<'a, R> where R: gfx::Resources {
     /// Constructor for a GlyphCache.
     pub fn new<D: gfx::Factory<R>>(font: &Path, device: &mut D)
                -> Result<GlyphCache<'static, R>, Error> {
        let freetype = match freetype::Library::init() {
            Ok(freetype) => freetype,
            Err(why) => return Err(Error::Freetype(why)),
        };
        let face = match freetype.new_face(font, 0) {
            Ok(face) => face,
            Err(why) => return Err(Error::Freetype(why)),
        };
        Ok(GlyphCache {
            face: face,
            empty_texture: try!(::Texture::empty(device).map_err(Error::Texture)),
            data: HashMap::new(),
        })
    }

    /// Generate all pending characters.
    pub fn update<D: gfx::Factory<R>>(&mut self, device: &mut D) {
        let ref empty_handle = self.empty_texture;
        for (&(size, ch), value) in self.data.iter_mut()
                .filter(|&(_, &mut Character { ref texture, .. })| texture == empty_handle) {
            self.face.set_pixel_sizes(0, size).unwrap();
            self.face.load_char(ch as usize, freetype::face::DEFAULT).unwrap();
            let glyph = self.face.glyph().get_glyph().unwrap();
            let bitmap_glyph = glyph.to_bitmap(
                freetype::render_mode::RenderMode::Normal, None)
                .unwrap();
            let bitmap = bitmap_glyph.bitmap();
            let texture = ::Texture::from_memory_alpha(
                device,
                bitmap.buffer(),
                bitmap.width() as u32,
                bitmap.rows() as u32);
            let glyph_size = glyph.advance();
            *value = Character {
                offset: [
                    bitmap_glyph.left() as f64,
                    bitmap_glyph.top() as f64
                ],
                size: [
                    (glyph_size.x >> 16) as f64,
                    (glyph_size.y >> 16) as f64
                ],
                texture: texture,
            };
        }
    }
}

impl<'a, R: gfx::Resources> graphics::character::CharacterCache for GlyphCache<'a, R> {
    type Texture = ::Texture<R>;

    fn character(&mut self, size: FontSize, ch: char) -> &Character<R> {
        match self.data.entry((size, ch)) {
            //returning `into_mut()' to work around lifetime issues
            Entry::Occupied(v) => v.into_mut(),
            Entry::Vacant(v) => {
                v.insert(graphics::character::Character {
                    offset: [0.0; 2],
                    size: [1.0; 2],
                    texture: self.empty_texture.clone()
                })
            },
        }
    }
}
