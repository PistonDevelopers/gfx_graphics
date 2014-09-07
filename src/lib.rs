#![crate_name = "gfx_graphics"]
#![deny(missing_doc)]
#![feature(phase)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

#[phase(plugin)]
extern crate gfx_macros;
extern crate device;
extern crate gfx;
extern crate graphics;
extern crate image;

pub use gfx2d::Gfx2d;
pub use gfx2d::RenderContext;
pub use texture::Texture;

mod gfx2d;
mod texture;
