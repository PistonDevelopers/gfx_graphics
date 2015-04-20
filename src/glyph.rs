//! Glyph caching

extern crate freetype as ft;

use std::path::Path;
use std::collections::hash_map::{ HashMap, Entry };
use graphics::character::{ CharacterCache, Character };
use graphics::types::FontSize;
use self::ft::render_mode::RenderMode;
use { gfx, Texture };

/// An enum to represent various possible run-time errors that may occur.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    /// An error happened when creating a gfx texture.
    Texture(gfx::tex::TextureError),
    /// An error happened with the FreeType library.
    Freetype(ft::error::Error)
}

impl From<gfx::tex::TextureError> for Error {
    fn from(tex_err: gfx::tex::TextureError) -> Self {
        Error::Texture(tex_err)
    }
}

impl From<ft::error::Error> for Error {
    fn from(ft_err: ft::error::Error) -> Self {
        Error::Freetype(ft_err)
    }
}

/// A struct used for caching rendered font.
pub struct GlyphCache<R> where R: gfx::Resources {
    /// The font face.
    pub face: ft::Face<'static>,
    empty_texture: Texture<R>,
    need_update: bool,
    data: HashMap<(FontSize, char), Character<Texture<R>>>
}

impl<R> GlyphCache<R> where R: gfx::Resources {
     /// Constructor for a GlyphCache.
    pub fn new<F>(font: &Path, factory: &mut F) -> Result<Self, Error>
        where F: gfx::Factory<R>
    {
        let freetype = try!(ft::Library::init());
        let face = try!(freetype.new_face(font, 0));
        Ok(GlyphCache {
            face: face,
            empty_texture: try!(Texture::empty(factory)),
            need_update: false,
            data: HashMap::new()
        })
    }

    /// Generate all pending characters.
    pub fn update<F>(&mut self, factory: &mut F) where F: gfx::Factory<R> {
        if self.need_update {
            let ref empty_texture = self.empty_texture;
            for (&(size, ch), value) in self.data.iter_mut().filter(
                |&(_, &mut Character { ref texture, .. })| texture == empty_texture
            ) {
                self.face.set_pixel_sizes(0, size).unwrap();
                self.face.load_char(ch as usize, ft::face::DEFAULT).unwrap();
                let glyph = self.face.glyph().get_glyph().unwrap();
                let bitmap_glyph = glyph.to_bitmap(RenderMode::Normal, None).unwrap();
                let glyph_size = glyph.advance();
                value.offset = [
                    bitmap_glyph.left() as f64,
                    bitmap_glyph.top() as f64
                ];
                value.size = [
                    (glyph_size.x >> 16) as f64,
                    (glyph_size.y >> 16) as f64
                ];
                if ch != ' ' {
                    let bitmap = bitmap_glyph.bitmap();
                    value.texture = Texture::from_memory_alpha(
                        factory,
                        bitmap.buffer(),
                        bitmap.width() as u32,
                        bitmap.rows() as u32
                    );
                }
            }
            self.need_update = false;
        }
    }
}

impl<R> CharacterCache for GlyphCache<R> where R: gfx::Resources {
    type Texture = Texture<R>;

    fn character(&mut self, size: FontSize, ch: char) -> &Character<Self::Texture> {
        match self.data.entry((size, ch)) {
            //returning `into_mut()' to get reference with 'a lifetime
            Entry::Occupied(v) => v.into_mut(),
            Entry::Vacant(v) => {
                if !self.need_update {
                    self.need_update = true;
                }
                v.insert(Character {
                    offset: [0.0; 2],
                    size: [0.0; 2],
                    texture: self.empty_texture.clone()
                })
            }
        }
    }
}
