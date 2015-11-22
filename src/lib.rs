#![deny(missing_docs)]

//! A Piston 2D graphics back-end using gfx-rs.

#[macro_use]
extern crate gfx;
extern crate gfx_texture;
extern crate graphics;
extern crate shaders_graphics2d as shaders;

pub use gfx_texture::*;

pub use back_end::{ Gfx2d, GfxGraphics };
pub use glyph::Error as GlyphError;
pub use glyph::GlyphCache;

mod back_end;
mod glyph;
