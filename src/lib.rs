#![crate_name = "gfx_graphics"]
#![deny(missing_doc)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

extern crate graphics;
extern crate gfx;
extern crate image;

pub use gfx2d::Gfx2d;
pub use texture::Texture;

mod gfx2d;
mod texture;
