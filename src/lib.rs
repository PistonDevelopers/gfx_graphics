#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! A [Piston 2D graphics](https://github.com/pistondevelopers/graphics) back-end using [gfx-rs](https://github.com/gfx-rs/gfx).
//!
//! Piston-Graphics is a generic library for 2D, part of the Piston ecosystem.
//! The generic abstraction creates triangles that are sent to the back-end.
//! Triangles are sent through the `Graphics` trait.
//!
//! ### How to use gfx_graphics
//!
//! If you are using the [piston_window](https://github.com/pistondevelopers/piston_window)
//! library, a `Gfx2d` object is created for you.
//! All you need to do is call `e.draw_2d(|c, g| { ... });`
//!
//! If you are not using a window wrapper, you need to create `Gfx2d` and `GfxGraphics`.
//!
//! 1. Create a `Gfx2d` object before the event loop
//! 2. Call `Gfx2d::draw` with `args.viewport()` from the render event.
//!
//! Example:
//!
//! ```ignore
//! let mut g2d = Gfx2d::new(api_version, &mut factory);
//! let mut events = window.events();
//! while let Some(e) = events.next(&mut window) {
//!     if let Some(args) = e.render_args() {
//!         g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
//!             ...
//!         }
//!     }
//! }
//! ```
//!
//! For a working example, see "examples/draw_state.rs".
//!
//! The closure `|c, g|` passes a `Context` and `&mut GfxGraphics` object.
//! `Context` contains viewport, transform and draw state information.
//!
//! When passing this to other functions, you usually write them as:
//!
//! ```ignore
//! fn draw_something<G: Graphics>(c: &Context, g: &mut G) {
//!     ...
//! }
//! ```
//!
//! The purpose is to make code reusable across Piston 2D back-ends.
//!
//! For more information, consult the documentation of Piston-Graphics.

#[macro_use]
extern crate gfx;
extern crate draw_state;
extern crate gfx_texture;
extern crate graphics;
extern crate shaders_graphics2d as shaders;
extern crate shader_version;

pub use gfx_texture::*;

pub use back_end::{ Gfx2d, GfxGraphics };
// pub use glyph::Error as GlyphError;
// pub use glyph::GlyphCache;

/// Stores textures for text rendering.
pub type GlyphCache<'a, F, R, C> =
    graphics::glyph_cache::rusttype::GlyphCache<'a, TextureContext<F, R, C>, Texture<R>>;

mod back_end;
