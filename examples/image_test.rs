extern crate piston;
extern crate graphics;
extern crate piston_window;
extern crate sdl2_window;
extern crate gfx_device_gl;
extern crate gfx_graphics;

use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use piston::window::{ WindowSettings, Size };
use piston_window::*;
use gfx_graphics::{ Texture, TextureSettings };
use sdl2_window::{ Sdl2Window, OpenGL };

fn main() {
    let window = Rc::new(RefCell::new(Sdl2Window::new(
        OpenGL::_3_2,
        WindowSettings::new(
            "gfx_graphics: image_test".to_string(),
            Size { width: 300, height: 300 }
        )
        .exit_on_esc(true)
    )));

    let events = PistonWindow::new(window, empty_app());
    let rust_logo = Texture::from_path(&mut events.canvas.borrow_mut().factory,
                                       &Path::new("./assets/rust.png"),
                                       &TextureSettings::new()).unwrap();
    for e in events {
        use graphics::*;

        e.draw_2d(|c, g| {
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
    }
}
