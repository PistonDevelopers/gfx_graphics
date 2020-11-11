extern crate graphics;
extern crate gfx_graphics;
extern crate piston;
extern crate glutin_window;
extern crate gfx;
extern crate gfx_device_gl;

use glutin_window::{GlutinWindow, OpenGL};
use gfx::traits::*;
use gfx::memory::Typed;
use gfx::format::{DepthStencil, Formatted, Srgba8};
use piston::window::{OpenGLWindow, Window, WindowSettings};
use piston::input::{AfterRenderEvent, RenderEvent, PressEvent};
use piston::event_loop::{Events, EventSettings, EventLoop};
use gfx_graphics::Gfx2d;
use graphics::draw_state::*;

fn main() {
    let opengl = OpenGL::V3_2;
    let (w, h) = (640, 480);
    let samples = 4;
    let mut window: GlutinWindow = WindowSettings::new("gfx_graphics: nested_clipping", [w, h])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .samples(samples)
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

    let mut encoder = factory.create_command_buffer().into();
    let mut g2d = Gfx2d::new(opengl, &mut factory);
    let mut events = Events::new(EventSettings::new().lazy(true));

    let increment = DrawState::new_increment();
    let inside_level1 = DrawState {
        blend: Some(Blend::Alpha),
        stencil: Some(Stencil::Inside(1)),
        scissor: None,
    };
    let inside_level2 = DrawState {
        blend: Some(Blend::Alpha),
        stencil: Some(Stencil::Inside(2)),
        scissor: None,
    };
    let inside_level3 = DrawState {
        blend: Some(Blend::Alpha),
        stencil: Some(Stencil::Inside(3)),
        scissor: None,
    };
    let mut clip = true;
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            use graphics::*;

            g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
                clear([0.8, 0.8, 0.8, 1.0], g);

                if clip {
                    Rectangle::new([1.0; 4])
                        .draw([10.0, 10.0, 200.0, 200.0],
                        &increment, c.transform, g);
                    Rectangle::new([1.0, 0.0, 0.0, 1.0])
                        .draw([10.0, 10.0, 200.0, 200.0],
                        &inside_level1, c.transform, g);

                    Rectangle::new([1.0; 4])
                        .draw([100.0, 100.0, 200.0, 200.0],
                        &increment, c.transform, g);
                    Rectangle::new([0.0, 0.0, 1.0, 1.0])
                        .draw([100.0, 100.0, 200.0, 200.0],
                        &inside_level2, c.transform, g);

                    Rectangle::new([1.0; 4])
                        .draw([100.0, 100.0, 200.0, 200.0],
                        &increment, c.transform, g);
                    Rectangle::new([0.0, 1.0, 0.0, 1.0])
                        .draw([50.0, 50.0, 200.0, 100.0],
                        &inside_level3, c.transform, g);
                } else {
                    Rectangle::new([1.0, 0.0, 0.0, 1.0])
                        .draw([10.0, 10.0, 200.0, 200.0],
                        &c.draw_state, c.transform, g);

                    Rectangle::new([0.0, 0.0, 1.0, 1.0])
                        .draw([100.0, 100.0, 200.0, 200.0],
                        &c.draw_state, c.transform, g);

                    Rectangle::new([0.0, 1.0, 0.0, 1.0])
                        .draw([50.0, 50.0, 200.0, 100.0],
                        &c.draw_state, c.transform, g);
                }
            });

            encoder.flush(&mut device);
        }
        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }
        if e.press_args().is_some() {
            clip = !clip;
        }
    }
}
