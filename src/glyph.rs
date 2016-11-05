//! Glyph caching

extern crate rusttype as rt;

use std::io;
use std::path::{Path};
use std::collections::hash_map::{ HashMap, Entry };
use graphics::character::{ CharacterCache, Character };
use graphics::types::{FontSize, Scalar};
use { gfx, Texture, TextureSettings };
use gfx::CombinedError;

/// An enum to represent various possible run-time errors that may occur.
#[derive(Debug)]
pub enum Error {
    /// An error happened when creating a gfx texture.
    Texture(CombinedError),
    /// An io error happened when reading font files.
    IoError(io::Error),
    /// No font was found in the file.
    NoFont,
}

impl From<CombinedError> for Error {
    fn from(tex_err: CombinedError) -> Self {
        Error::Texture(tex_err)
    }
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Self {
        Error::IoError(io_error)
    }
}

/// A struct used for caching a rendered font.
pub struct GlyphCache<R, F> where R: gfx::Resources {
    /// The font.
    pub font: rt::Font<'static>,
    factory: F,
    // Maps from fontsize and character to offset, size and texture.
    data: HashMap<(FontSize, char), ([Scalar; 2], [Scalar; 2], Texture<R>)>
}

impl<R, F> GlyphCache<R, F> where R: gfx::Resources {
     /// Constructor for a GlyphCache.
    pub fn new<P>(font_path: P, factory: F) -> Result<Self, Error>
        where P: AsRef<Path>    {

        use std::io::Read;
        use std::fs::File;

        let mut file = try!(File::open(font_path));
        let mut file_buffer = Vec::new();
        try!(file.read_to_end(&mut file_buffer));

        let collection = rt::FontCollection::from_bytes(file_buffer);
        let font = match collection.into_font() {
            Some(font) => font,
            None => return Err(Error::NoFont),
        };

        Ok(GlyphCache {
            font: font,
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
        let size = ((size as f32) * 1.333).round() as u32 ; // convert points to pixels

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
                let glyph = self.font.glyph(ch).unwrap(); // this is only None for invalid GlyphIds, but char is converted to a Codepoint which must result in a glyph.
                let scale = rt::Scale::uniform(size as f32);
                let mut glyph = glyph.scaled(scale);

                // some fonts do not contain glyph zero as fallback, instead try U+FFFD.
                if glyph.id() == rt::GlyphId(0) && glyph.shape().is_none() {
                    glyph = self.font.glyph('\u{FFFD}').unwrap().scaled(scale);
                }

                let h_metrics = glyph.h_metrics();
                let bounding_box = glyph.exact_bounding_box().unwrap_or(rt::Rect{min: rt::Point{x: 0.0, y: 0.0}, max: rt::Point{x: 0.0, y: 0.0} });
                let glyph = glyph.positioned(rt::point(0.0, 0.0));
                let pixel_bounding_box = glyph.pixel_bounding_box().unwrap_or(rt::Rect{min: rt::Point{x: 0, y: 0}, max: rt::Point{x: 0, y: 0} });
                let pixel_bb_width = pixel_bounding_box.width() + 2;
                let pixel_bb_height = pixel_bounding_box.height() + 2;

                let mut image_buffer = Vec::<u8>::new();
                image_buffer.resize((pixel_bb_width * pixel_bb_height) as usize, 0);
                glyph.draw(|x, y, v| {
                   let pos = ((x+1) + (y+1) * (pixel_bb_width as u32)) as usize;
                   image_buffer[pos] = (255.0 * v) as u8;
                });

                let &mut (offset, size, ref texture) = v.insert((
                    [
                        bounding_box.min.x as Scalar + 1.0,
                        -pixel_bounding_box.min.y as Scalar + 1.0,
                    ],
                    [
                        h_metrics.advance_width as Scalar,
                        0 as Scalar,
                    ],
                    {
                        if pixel_bb_width == 0 || pixel_bb_height == 0 {
                            Texture::empty(&mut self.factory)
                                    .unwrap()
                        } else {
                            Texture::from_memory_alpha(
                                &mut self.factory,
                                &image_buffer,
                                pixel_bb_width as u32,
                                pixel_bb_height as u32,
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
