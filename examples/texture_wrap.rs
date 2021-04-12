extern crate gfx;
extern crate gfx_graphics;
extern crate glutin_window;
extern crate graphics;
extern crate image as im;
extern crate piston;

use gfx::format::{DepthStencil, Formatted, Srgba8};
use gfx::memory::Typed;
use gfx_graphics::{Gfx2d, Texture, TextureContext, TextureSettings, Wrap};
use glutin_window::{GlutinWindow, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::{OpenGLWindow, Window, WindowSettings};
use std::path::Path;

fn main() {
    println!("Press U to change the texture wrap mode for the u coordinate");
    println!("Press V to change the texture wrap mode for the v coordinate");

    let opengl = OpenGL::V3_2;
    let samples = 4;
    let (w, h) = (640, 480);
    let mut window: GlutinWindow = WindowSettings::new("gfx_graphics: texture_wrap", [w, h])
        .exit_on_esc(true)
        .samples(samples)
        .graphics_api(opengl)
        .build()
        .unwrap();
    let (mut device, mut factory) =
        gfx_device_gl::create(|s| window.get_proc_address(s) as *const std::os::raw::c_void);

    // Set up wrap modes
    let wrap_modes = [
        Wrap::ClampToEdge,
        Wrap::ClampToBorder,
        Wrap::Repeat,
        Wrap::MirroredRepeat,
    ];
    let mut ix_u = 0;
    let mut ix_v = 0;
    let mut texture_settings = TextureSettings::new();
    texture_settings.set_border_color([0.0, 0.0, 0.0, 1.0]);

    // Set up texture
    let path = Path::new("./assets/rust.png");
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
    let mut rust_logo = Texture::from_image(&mut texture_context, &img, &texture_settings).unwrap();

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
    let mut events = Events::new(EventSettings::new().lazy(true));
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            use graphics::*;
            g2d.draw(
                &mut encoder,
                &output_color,
                &output_stencil,
                args.viewport(),
                |_c, g| {
                    clear([1.0; 4], g);
                    let points = [[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]];
                    // (0, 1, 2) and (0, 2, 3)
                    let uvs = [
                        [4.0, 0.0],
                        [0.0, 0.0],
                        [0.0, 4.0],
                        [4.0, 0.0],
                        [0.0, 4.0],
                        [4.0, 4.0],
                    ];
                    let mut verts = [[0.0, 0.0]; 6];
                    let indices_points: [usize; 6] = [0, 1, 2, 0, 2, 3];
                    for (ixv, &ixp) in (0..6).zip(indices_points.iter()) {
                        verts[ixv] = points[ixp];
                    }
                    g.tri_list_uv(&DrawState::new_alpha(), &[1.0; 4], &rust_logo, |f| {
                        f(&verts, &uvs)
                    });
                },
            );
            encoder.flush(&mut device);
        }

        if let Some(Button::Keyboard(Key::U)) = e.press_args() {
            ix_u = (ix_u + 1) % wrap_modes.len();
            texture_settings.set_wrap_u(wrap_modes[ix_u]);
            rust_logo = Texture::from_image(&mut texture_context, &img, &texture_settings).unwrap();
            println!(
                "Changed texture wrap mode for u coordinate to: {:?}",
                wrap_modes[ix_u]
            );
        }

        if let Some(Button::Keyboard(Key::V)) = e.press_args() {
            ix_v = (ix_v + 1) % wrap_modes.len();
            texture_settings.set_wrap_v(wrap_modes[ix_v]);
            rust_logo = Texture::from_image(&mut texture_context, &img, &texture_settings).unwrap();
            println!(
                "Changed texture wrap mode for v coordinate to: {:?}",
                wrap_modes[ix_v]
            );
        }
    }
}
