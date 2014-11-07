
#![feature(globs)]

extern crate shader_version;
extern crate graphics;
extern crate event;
extern crate sdl2_window;
extern crate gfx;
extern crate gfx_graphics;
extern crate sdl2;

use std::cell::RefCell;
use event::{
    Events,
    Window,
    WindowSettings,
};
use gfx::{Device, DeviceHelper};
use gfx_graphics::{
    G2D,
    Texture,
};
use sdl2_window::Sdl2Window;

fn main() {
    let opengl = shader_version::opengl::OpenGL_3_2;
    let window = Sdl2Window::new(
        opengl,
        WindowSettings {
            title: "gfx_graphics: imagetest".to_string(),
            size: [300, 300],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );

    let mut device = gfx::GlDevice::new(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });
    let (w, h) = window.get_size();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let mut renderer = device.create_renderer();

    let image = Texture::from_path(&mut device, 
        &Path::new("rust.png")).unwrap();
    let mut g2d = G2D::new(&mut device);
    let window = RefCell::new(window);
    for e in Events::new(&window) {
        use event::RenderEvent;
        e.render(|_| {
            g2d.draw(&mut renderer, &frame, |c, g| {
                use graphics::*;
                c.rgb(1.0, 1.0, 1.0).draw(g);
                c.rect(0.0, 0.0, 100.0, 100.0).rgb(1.0, 0.0, 0.0).draw(g);
                c.rect(50.0, 50.0, 100.0, 100.0).rgba(0.0, 1.0, 0.0, 0.3).draw(g);
                c.trans(100.0, 100.0).image(&image).draw(g);
            });

            device.submit(renderer.as_buffer());
            renderer.reset();
       });
    }
}
