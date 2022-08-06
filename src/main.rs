#[allow(no_snake_name)]
mod app;
use crate::app::App;
mod beatmap;
use crate::beatmap::BeatMap;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut beatmap = BeatMap::new("Team Grimoire - C18H27NO3 ([Shana Lesus]) [Alexey's 4K BASIC].osu");
    // Create a new game and run it.
    // let mut app = App {
    //     gl: GlGraphics::new(opengl),
    //     rotation: 0.0,
    // };

    // let mut events = Events::new(EventSettings::new());
    // while let Some(e) = events.next(&mut window) {
    //     if let Some(args) = e.render_args() {
    //         app.render(&args);
    //     }

    //     if let Some(args) = e.update_args() {
    //         app.update(&args);
    //     }
    // }
}
