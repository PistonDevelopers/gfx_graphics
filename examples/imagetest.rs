#![feature(old_path)]

extern crate piston;
extern crate shader_version;
extern crate graphics;
extern crate sdl2_window;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_graphics;

use piston::quack::Get;
use std::cell::RefCell;
use gfx::traits::*;
use gfx_graphics::{
    Gfx2d,
    Texture,
};
use sdl2_window::Sdl2Window;

fn main() {
    let opengl = shader_version::OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        piston::window::WindowSettings {
            title: "gfx_graphics: imagetest".to_string(),
            size: [300, 300],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );

    let mut device = gfx_device_gl::GlDevice::new(Sdl2Window::get_proc_address);
    let piston::window::Size([w, h]) = window.get();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let mut renderer = device.create_renderer();

    let image = Texture::from_path(&mut device,
        &Path::new("./assets/rust.png")).unwrap();
    let mut g2d = Gfx2d::new(&mut device);
    let window = RefCell::new(window);
    for e in piston::events(&window) {
        use piston::event::RenderEvent;
        if let Some(_) = e.render_args() {
            use graphics::RelativeTransform;

            g2d.draw(&mut renderer, &frame, |c, g| {
                let transform = c.transform.trans(100.0, 100.0);

                graphics::clear([1.0; 4], g);
                graphics::Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([0.0, 0.0, 100.0, 100.0],
                          &c.draw_state,
                          c.transform,
                          g);
                graphics::Rectangle::new([0.0, 1.0, 0.0, 0.3])
                    .draw([50.0, 50.0, 100.0, 100.0],
                          &c.draw_state,
                          c.transform,
                          g);
                graphics::image(&image, transform, g);
            });

            device.submit(renderer.as_buffer());
            renderer.reset();
       }
    }
}
