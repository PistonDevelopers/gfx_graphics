extern crate glutin_window;
extern crate piston;
extern crate graphics;
extern crate gfx_graphics;
extern crate find_folder;
extern crate gfx;
extern crate gfx_device_gl;

use gfx::traits::*;
use glutin_window::{GlutinWindow, OpenGL};
use piston::window::{OpenGLWindow, Window, WindowSettings};
use piston::event_loop::Events;
use graphics::draw_state::preset::blend::*;
use graphics::*;
use piston::input::*;
use gfx_graphics::{Flip, Gfx2d, Texture, TextureSettings};

fn main() {
    println!("Press A to change blending");
    println!("Press S to change clip inside/out");

    let opengl = OpenGL::V3_2;
    let samples = 4;
    let mut window: GlutinWindow = WindowSettings::new(
            "piston: draw_state",
            [600, 600]
        )
        .exit_on_esc(true)
        .samples(samples)
        .opengl(opengl)
        .build()
        .unwrap();

    let (mut device, mut factory) = gfx_device_gl::create(|s|
        window.get_proc_address(s) as *const std::os::raw::c_void);
    // create the main color/depth targets
    let draw_size = window.draw_size();
    let aa = samples as gfx::tex::NumSamples;
    let dim = (draw_size.width as u16, draw_size.height as u16, 1, aa.into());
    let (output_color, output_stencil) = gfx_device_gl::create_main_targets(dim);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let blends = [ADD, ALPHA, INVERT, MULTIPLY];
    let mut blend = 0;
    let mut clip_inside = true;
    let rust_logo = Texture::from_path(&mut factory,
                                       assets.join("rust.png"),
                                       Flip::None,
                                       &TextureSettings::new()).unwrap();

    let mut encoder = factory.create_encoder();
    let mut g2d = Gfx2d::new(opengl, &mut factory);
    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
                clear([0.8, 0.8, 0.8, 1.0], g);
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);

                let draw_state = c.draw_state.blend(blends[blend]);
                Rectangle::new([0.5, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0], &draw_state, c.transform, g);

                let transform = c.transform.trans(100.0, 100.0);
                // Compute clip rectangle from upper left corner.
                let (clip_x, clip_y, clip_w, clip_h) = (100, 100, 100, 100);
                let (clip_x, clip_y, clip_w, clip_h) =
                    (clip_x, c.viewport.unwrap().draw_size[1] - clip_y - clip_h, clip_w, clip_h);
                let clipped = c.draw_state.scissor([clip_x, clip_y, clip_w, clip_h]);
                Image::new().draw(&rust_logo, &clipped, transform, g);

                let transform = c.transform.trans(200.0, 200.0);
                Ellipse::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 50.0, 50.0], clip_draw_state(), transform, g);
                Image::new().draw(&rust_logo,
                    if clip_inside { inside_draw_state() }
                    else { outside_draw_state() },
                    transform, g);
            });
            device.submit(encoder.as_buffer());
        }

        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            blend = (blend + 1) % blends.len();
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
