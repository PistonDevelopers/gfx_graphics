#![deny(missing_docs)]

//! The implementation of a Rust-Graphics back-end using gfx-rs.

#[macro_use]
extern crate gfx_macros;
extern crate gfx;
extern crate graphics;
extern crate image;
extern crate gfx_texture;

pub use gfx_texture::Texture;

pub use g2d::G2D;
pub use g2d::GraphicsBackEnd;

mod g2d;
