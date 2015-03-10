#![deny(missing_docs)]
#![feature(plugin, old_path, std_misc, custom_attribute)]
#![plugin(gfx_macros)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

extern crate gfx;
extern crate gfx_texture;
extern crate graphics;
extern crate image;
extern crate freetype;

pub use gfx_texture::Texture;

pub use back_end::Gfx2d;
pub use back_end::GfxGraphics;
pub use back_end::Gfx2d as G2D;
pub use back_end::GfxGraphics as GraphicsBackEnd;
pub use glyph::GlyphCache;

mod back_end;
mod glyph;
