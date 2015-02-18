#![deny(missing_docs)]
#![feature(plugin, path, std_misc)]
#![plugin(gfx_macros)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

#[macro_use]
extern crate gfx_macros;
extern crate gfx;
extern crate gfx_texture;
extern crate graphics;
extern crate image;
extern crate freetype;

pub use gfx_texture::Texture;

pub use g2d::G2D;
pub use g2d::GraphicsBackEnd;
pub use glyph::GlyphCache;

mod g2d;
mod glyph;
