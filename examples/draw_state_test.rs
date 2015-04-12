extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate gfx_device_gl;
extern crate gfx_graphics;

use std::path::Path;
use piston::window::{ Window, WindowSettings, Size, OpenGLWindow };
use piston::event::*;
use gfx_graphics::gfx::traits::*;
use gfx_graphics::{ Gfx2d, Texture, TextureSettings };
use sdl2_window::{ Sdl2Window, OpenGL };
use graphics::draw_state::BlendPreset;

fn main() {
    println!("Press A to change blending");

    let mut window = Sdl2Window::new(
        OpenGL::_3_2,
        WindowSettings::new(
            "gfx_graphics: draw_state_test".to_string(),
            Size { width: 600, height: 600 }
        )
        .exit_on_esc(true)
    );

    let (mut device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s));
    let size = window.size();
    let output = factory.make_fake_output(size.width as u16, size.height as u16);
    let mut renderer = factory.create_renderer();

    let rust_logo = Texture::from_path(&mut factory,
                                       &Path::new("./assets/rust.png"),
                                       &TextureSettings::new()).unwrap();
    let mut g2d = Gfx2d::new(&mut device, &mut factory);
    let mut blend = BlendPreset::Alpha;
    for e in window.events() {
        use piston::input::*;

        if let Some(args) = e.render_args() {
            use graphics::*;

            g2d.draw(&mut renderer, &output, args.viewport(), |c, g| {
                clear([0.8, 0.8, 0.8, 1.0], g);
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0],
                          &c.draw_state,
                          c.transform,
                          g);

                let draw_state = c.draw_state.blend(blend);
                Rectangle::new([0.5, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0],
                          &draw_state,
                          c.transform,
                          g);

                let transform = c.transform.trans(100.0, 100.0);
                // Compute clip rectangle from upper left corner.
                let (clip_x, clip_y, clip_w, clip_h) = (100, 100, 100, 100);
                let (clip_x, clip_y, clip_w, clip_h) =
                    (clip_x, args.height as u16 - clip_y - clip_h, clip_w, clip_h);
                let clipped = c.draw_state.scissor(clip_x, clip_y, clip_w, clip_h);
                Image::new().draw(&rust_logo, &clipped, transform, g);

                let transform = c.transform.trans(200.0, 200.0);
                Ellipse::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 50.0, 50.0],
                          clip_draw_state(),
                          transform,
                          g);
                Image::new().draw(&rust_logo, inside_draw_state(), transform, g);
            });

            device.submit(renderer.as_buffer());
            renderer.reset();
        }

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            blend = match blend {
                BlendPreset::Alpha => BlendPreset::Add,
                BlendPreset::Add => BlendPreset::Multiply,
                BlendPreset::Multiply => BlendPreset::Invert,
                BlendPreset::Invert => BlendPreset::Alpha,
            };
            println!("Changed blending to {:?}", blend);
        }

        if let Some(_) = e.after_render_args() {
            device.after_frame();
            factory.cleanup();
        }
    }
}
