extern crate piston;
extern crate graphics;
extern crate gfx;
extern crate gfx_graphics;
extern crate glutin_window;
extern crate image as im;

use std::path::Path;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{OpenGLWindow, Window, WindowSettings};
use gfx::format::{DepthStencil, Formatted, Srgba8};
use gfx::memory::Typed;
use gfx::Device;
use gfx_graphics::*;
use glutin_window::{OpenGL, GlutinWindow};

fn main() {
    let opengl = OpenGL::V3_2;
    let samples = 4;
    let mut window: GlutinWindow = WindowSettings::new("gfx_graphics: colored_image_test", [300, 300])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .samples(samples)
        .build()
        .unwrap();
    let (mut device, mut factory) =
        gfx_device_gl::create(|s| window.get_proc_address(s) as *const std::os::raw::c_void);

    // Set up texture
    let path = Path::new("./assets/rust-white.png");
    let img = match im::open(path) {
        Ok(img) => img,
        Err(e) => {
            panic!("Could not load '{:?}': {:?}", path.file_name().unwrap(), e);
        }
    };
    let img = match img {
        im::DynamicImage::ImageRgba8(img) => img,
        x => x.to_rgba8(),
    };
    let mut texture_context = TextureContext {
        factory: factory.clone(),
        encoder: factory.create_command_buffer().into(),
    };
    let rust_logo = Texture::from_image(&mut texture_context, &img, &TextureSettings::new()).unwrap();

    let mut encoder = factory.create_command_buffer().into();
    let draw_size = window.draw_size();
    let dim = (
        draw_size.width as u16,
        draw_size.height as u16,
        1,
        (samples as gfx::texture::NumSamples).into(),
    );
    let color_format = <Srgba8 as Formatted>::get_format();
    let depth_format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim, color_format.0, depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);

    let mut g2d = Gfx2d::new(opengl, &mut factory);
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        use graphics::*;

        if let Some(args) = e.render_args() {
            g2d.draw(
                &mut encoder,
                &output_color,
                &output_stencil,
                args.viewport(),
                |c, g|
            {
                use graphics::triangulation::{tx, ty};

                let transform = c.transform.trans(0.0, 0.0);

                let tr = |p: [f64; 2]| [tx(transform, p[0], p[1]), ty(transform, p[0], p[1])];

                clear([1.0; 4], g);
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);
                Rectangle::new([0.0, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0], &c.draw_state, c.transform, g);
                g.tri_list_uv_c(&c.draw_state, &rust_logo, |f| {
                    (f)(
                        &[
                            tr([0.0, 0.0]),
                            tr([300.0, 0.0]),
                            tr([0.0, 300.0]),

                            tr([300.0, 0.0]),
                            tr([0.0, 300.0]),
                            tr([300.0, 300.0]),
                        ],
                        &[
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [0.0, 1.0],

                            [1.0, 0.0],
                            [0.0, 1.0],
                            [1.0, 1.0],
                        ],
                        &[
                            [1.0, 0.0, 0.0, 1.0],
                            [0.0, 1.0, 0.0, 1.0],
                            [0.0, 0.0, 1.0, 1.0],

                            [0.0, 00.0, 0.0, 1.0],
                            [0.0, 00.0, 0.0, 1.0],
                            [0.0, 00.0, 0.0, 1.0],
                        ]
                    )
                });
            });
            encoder.flush(&mut device);
        }
        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }
    }
}
