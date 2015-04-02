#![deny(missing_docs)]
#![feature(plugin, custom_attribute)]
#![plugin(gfx_macros)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

extern crate gfx as gfx_lib;
extern crate gfx_texture;
extern crate graphics;
extern crate image;
extern crate freetype;

/// Reexports from gfx's crate.
pub mod gfx {
    pub use gfx_lib::traits;
    pub use gfx_lib::Frame;
}
pub use gfx_texture::Texture;
pub use gfx_texture::Settings as TextureSettings;

pub use back_end::{ Gfx2d, GfxGraphics };
pub use Gfx2d as G2D;
pub use GfxGraphics as GraphicsBackEnd;
pub use glyph::GlyphCache;

mod back_end;
mod glyph;
mod shaders;
