//! Glyph caching

extern crate freetype as ft;

use std::path::Path;
use std::collections::hash_map::{ HashMap, Entry };
use graphics::character::{ CharacterCache, Character };
use graphics::types::{FontSize, Scalar};
use self::ft::render_mode::RenderMode;
use { gfx, Texture, TextureSettings };
use gfx::core::factory::CombinedError;

/// An enum to represent various possible run-time errors that may occur.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// An error happened when creating a gfx texture.
    Texture(CombinedError),
    /// An error happened with the FreeType library.
    Freetype(ft::error::Error)
}

impl From<CombinedError> for Error {
    fn from(tex_err: CombinedError) -> Self {
        Error::Texture(tex_err)
    }
}

impl From<ft::error::Error> for Error {
    fn from(ft_err: ft::error::Error) -> Self {
        Error::Freetype(ft_err)
    }
}

/// A struct used for caching rendered font.
pub struct GlyphCache<R, F> where R: gfx::Resources {
    /// The font face.
    pub face: ft::Face<'static>,
    factory: F,
    // Maps from fontsize and character to offset, size and texture.
    data: HashMap<(FontSize, char), ([Scalar; 2], [Scalar; 2], Texture<R>)>
}

impl<R, F> GlyphCache<R, F> where R: gfx::Resources {
     /// Constructor for a GlyphCache.
    pub fn new<P>(font: P, factory: F) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let freetype = try!(ft::Library::init());
        let face = try!(freetype.new_face(font.as_ref(), 0));
        Ok(GlyphCache {
            face: face,
            factory: factory,
            data: HashMap::new(),
        })
    }
}

impl<R, F> CharacterCache for GlyphCache<R, F> where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    type Texture = Texture<R>;

    fn character<'a>(
        &'a mut self,
        size: FontSize,
        ch: char
    ) -> Character<'a, Self::Texture> {
        match self.data.entry((size, ch)) {
            //returning `into_mut()' to get reference with 'a lifetime
            Entry::Occupied(v) => {
                let &mut (offset, size, ref texture) = v.into_mut();
                Character {
                    offset: offset,
                    size: size,
                    texture: texture
                }
            }
            Entry::Vacant(v) => {
                self.face.set_pixel_sizes(0, size).unwrap();
                self.face.load_char(ch as usize, ft::face::DEFAULT).unwrap();
                let glyph = self.face.glyph().get_glyph().unwrap();
                let bitmap_glyph = glyph.to_bitmap(RenderMode::Normal, None).unwrap();
                let glyph_size = [glyph.advance_x(), glyph.advance_y()];
                let &mut (offset, size, ref texture) = v.insert((
                    [
                        bitmap_glyph.left() as f64,
                        bitmap_glyph.top() as f64
                    ],
                    [
                        (glyph_size[0] >> 16) as f64,
                        (glyph_size[1] >> 16) as f64
                    ],
                    {
                        let bitmap = bitmap_glyph.bitmap();
                        if bitmap.width() == 0 || bitmap.rows() == 0 {
                            Texture::empty(&mut self.factory)
                                    .unwrap()
                        } else {
                            Texture::from_memory_alpha(
                                &mut self.factory,
                                bitmap.buffer(),
                                bitmap.width() as u32,
                                bitmap.rows() as u32,
                                &TextureSettings::new()
                            ).unwrap()
                        }
                    },
                ));
                Character {
                    offset: offset,
                    size: size,
                    texture: texture
                }
            }
        }
    }
}
