
#![feature(globs)]

extern crate graphics;
extern crate piston;
extern crate sdl2_game_window;
extern crate gfx;
extern crate gfx_graphics;

use gfx::{Device, DeviceHelper};
use gfx_graphics::{
    Gfx2d,
    RenderContext,
    Texture,
};
use sdl2_game_window::WindowSDL2;
use graphics::*;
use piston::{
    AssetStore,
    EventIterator,
    EventSettings,
    WindowSettings,
    Render,
};

fn main() {
    let opengl = piston::shader_version::opengl::OpenGL_3_2;
    let mut window = WindowSDL2::new(
        opengl,
        WindowSettings {
            title: "Image".to_string(),
            size: [300, 300],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );

    let (mut device, frame) = window.gfx();
    let mut renderer = device.create_renderer();

    let asset_store = AssetStore::from_folder("../bin/assets");

    let image = asset_store.path("rust-logo.png").unwrap();
    let image = Texture::from_path(&mut device, &image).unwrap();
    let event_settings = EventSettings {
            updates_per_second: 120,
            max_frames_per_second: 60,
        };
    let mut gfx2d = Gfx2d::new(&mut device);
    for e in EventIterator::new(&mut window, &event_settings) {
        match e {
            Render(args) => {
                {
                    let ref mut g = RenderContext::new(&mut renderer, &frame, &mut gfx2d);
                    let c = Context::abs(args.width as f64, args.height as f64);
                    c.rgb(1.0, 1.0, 1.0).draw(g);
                    c.rect(0.0, 0.0, 100.0, 100.0).rgb(1.0, 0.0, 0.0).draw(g);
                    c.rect(50.0, 50.0, 100.0, 100.0).rgba(0.0, 1.0, 0.0, 0.3).draw(g);
                    c.trans(100.0, 100.0).image(&image).draw(g);
                }

                device.submit(renderer.as_buffer());
                renderer.reset();
            },
            _ => {},
        }
    }
}
