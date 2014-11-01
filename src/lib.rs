#![deny(missing_docs)]
#![feature(phase)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

#[phase(plugin)]
extern crate gfx_macros;
extern crate device;
extern crate gfx;
extern crate graphics;
extern crate image;
extern crate gfx_texture;

pub use gfx_texture::Texture;

pub use g2d::G2D;
pub use g2d::GraphicsBackEnd;

mod g2d;
