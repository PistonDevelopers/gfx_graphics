extern crate glutin_window;
extern crate piston;
extern crate graphics;
extern crate gfx_graphics;
extern crate find_folder;
extern crate gfx;
extern crate gfx_device_gl;

use gfx::traits::*;
use gfx::format::{DepthStencil, Formatted, Srgba8};
use gfx::memory::Typed;
use glutin_window::{GlutinWindow, OpenGL};
use piston::window::{OpenGLWindow, Window, WindowSettings};
use piston::event_loop::{Events, EventSettings, EventLoop};
use graphics::draw_state::Blend;
use graphics::*;
use piston::input::*;
use gfx_graphics::{Flip, Gfx2d, Texture, TextureSettings, TextureContext};

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
        .graphics_api(opengl)
        .build()
        .unwrap();

    let (mut device, mut factory) = gfx_device_gl::create(|s|
        window.get_proc_address(s) as *const std::os::raw::c_void);

    // Create the main color/depth targets.
    let draw_size = window.draw_size();
    let aa = samples as gfx::texture::NumSamples;
    let dim = (draw_size.width as u16, draw_size.height as u16, 1, aa.into());
    let color_format = <Srgba8 as Formatted>::get_format();
    let depth_format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim,
                                               color_format.0,
                                               depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let blends = [Blend::Alpha, Blend::Add, Blend::Invert, Blend::Multiply, Blend::Lighter];
    let mut blend = 0;
    let mut clip_inside = true;
    let mut texture_context = TextureContext {
        factory: factory.clone(),
        encoder: factory.create_command_buffer().into(),
    };
    let rust_logo = Texture::from_path(&mut texture_context,
                                       assets.join("rust.png"),
                                       Flip::None,
                                       &TextureSettings::new()).unwrap();

    let mut encoder = factory.create_command_buffer().into();
    let mut g2d = Gfx2d::new(opengl, &mut factory);
    let mut events = Events::new(EventSettings::new().lazy(true));
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
                // Clip rectangle from upper left corner.
                let clipped = c.draw_state.scissor([100, 100, 100, 100]);
                Image::new().draw(&rust_logo, &clipped, transform, g);

                let transform = c.transform.trans(200.0, 200.0);
                Ellipse::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 50.0, 50.0], &DrawState::new_clip(), transform, g);
                Image::new().draw(&rust_logo,
                    &if clip_inside { DrawState::new_inside() }
                    else { DrawState::new_outside() },
                    transform, g);
            });
            encoder.flush(&mut device);
        }

        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }

        if let Some(Button::Keyboard(Key::A)) = e.press_args() {
            blend = (blend + 1) % blends.len();
            println!("Changed blending to {:?}", blends[blend]);
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
