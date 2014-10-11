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

pub use g2d::G2D;
pub use g2d::GraphicsBackEnd;
pub use texture::Texture;

mod g2d;
mod texture;
