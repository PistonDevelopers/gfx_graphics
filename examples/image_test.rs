extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate gfx_device_gl;
extern crate gfx_graphics;

use std::path::Path;
use piston::window::{ Window, WindowSettings, Size, OpenGLWindow };
use piston::event::*;
use gfx_graphics::gfx::traits::*;
use gfx_graphics::{ Gfx2d, Texture, gfx, TextureSettings };
use sdl2_window::{ Sdl2Window, OpenGL };

fn main() {
    let mut window = Sdl2Window::new(
        OpenGL::_3_2,
        WindowSettings::new(
            "gfx_graphics: image_test".to_string(),
            Size { width: 300, height: 300 }
        )
        .exit_on_esc(true)
    );

    let (mut device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s));
    let size = window.size();
    let frame = gfx::Frame::new(size.width as u16, size.height as u16);
    let mut renderer = factory.create_renderer();

    let rust_logo = Texture::from_path(&mut factory,
                                       &Path::new("./assets/rust.png"),
                                       &TextureSettings::new()).unwrap();
    let mut g2d = Gfx2d::new(&mut device, &mut factory);
    for e in window.events() {
        if let Some(args) = e.render_args() {
            use graphics::*;

            g2d.draw(&mut renderer, &frame, args.viewport(), |c, g| {
                let transform = c.transform.trans(100.0, 100.0);

                clear([1.0; 4], g);
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0],
                          &c.draw_state,
                          c.transform,
                          g);
                Rectangle::new([0.0, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0],
                          &c.draw_state,
                          c.transform,
                          g);
                image(&rust_logo, transform, g);
            });

            device.submit(renderer.as_buffer());
            renderer.reset();
        }

        if let Some(_) = e.after_render_args() {
            device.after_frame();
            factory.cleanup();
        }
    }
}
