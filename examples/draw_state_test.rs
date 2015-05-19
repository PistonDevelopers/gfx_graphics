extern crate piston;
extern crate graphics;
extern crate piston_window;
extern crate sdl2_window;
extern crate gfx_device_gl;
extern crate gfx_graphics;

use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use piston::window::WindowSettings;
use piston::event::*;
use gfx_graphics::{ Texture, TextureSettings };
use sdl2_window::{ Sdl2Window, OpenGL };
use graphics::draw_state::BlendPreset;
use piston_window::*;

fn main() {
    println!("Press A to change blending");
    println!("Press S to change clip inside/out");

    let window = Rc::new(RefCell::new(
        Sdl2Window::new(
            OpenGL::_3_2,
            WindowSettings::new(
                "gfx_graphics: draw_state_test",
                [600, 600]
            )
            .exit_on_esc(true)
            .samples(4)
        )
    ));

    let events = PistonWindow::new(window, empty_app());
    let mut blend = BlendPreset::Alpha;
    let mut clip_inside = true;
    let ref mut factory = events.device.borrow().spawn_factory();
    let rust_logo = Texture::from_path(factory,
                                       &Path::new("./assets/rust.png"),
                                       &TextureSettings::new()).unwrap();
    for e in events {
        use piston::input::*;
        use graphics::*;

        e.draw_2d(|c, g| {
            clear([0.8, 0.8, 0.8, 1.0], g);
            g.clear_stencil(0);
            Rectangle::new([1.0, 0.0, 0.0, 1.0])
                .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);

            let draw_state = c.draw_state.blend(blend);
            Rectangle::new([0.5, 1.0, 0.0, 0.3])
                .draw([50.0, 50.0, 100.0, 100.0], &draw_state, c.transform, g);

            let transform = c.transform.trans(100.0, 100.0);
            // Compute clip rectangle from upper left corner.
            let (clip_x, clip_y, clip_w, clip_h) = (100, 100, 100, 100);
            let (clip_x, clip_y, clip_w, clip_h) =
                (clip_x, c.viewport.unwrap().draw_size[1] as u16 - clip_y - clip_h, clip_w, clip_h);
            let clipped = c.draw_state.scissor(clip_x, clip_y, clip_w, clip_h);
            Image::new().draw(&rust_logo, &clipped, transform, g);

            let transform = c.transform.trans(200.0, 200.0);
            Ellipse::new([1.0, 0.0, 0.0, 1.0])
                .draw([0.0, 0.0, 50.0, 50.0], clip_draw_state(), transform, g);
            Image::new().draw(&rust_logo,
                if clip_inside { inside_draw_state() }
                else { outside_draw_state() },
                transform, g);
        });

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            blend = match blend {
                BlendPreset::Alpha => BlendPreset::Add,
                BlendPreset::Add => BlendPreset::Multiply,
                BlendPreset::Multiply => BlendPreset::Invert,
                BlendPreset::Invert => BlendPreset::Alpha,
            };
            println!("Changed blending to {:?}", blend);
        }

        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            clip_inside = !clip_inside;
            if clip_inside {
                println!("Changed to clip inside");
            } else {
                println!("Changed to clip outside");
            }
        }
    }
}
