#[allow(non_snake_case)]
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use graphics::{Rectangle, Graphics, image};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use graphics::{Image, clear, draw_state::DrawState, rectangle, line};
use graphics::rectangle::square;
use core::f64;
use std::path::Path;
use lazy_static::lazy_static;


use crate::beatmap::{BeatMap, HitObject};


static WINDOW_LENGTH_MS: u64 = 512;

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub beatmap: BeatMap,
}

impl App {
    pub fn render(&mut self, args: &RenderArgs, time: u64) {

	// // Create the image object and attach a square Rectangle object inside.
	//let image   = Image::new().rect([0.0,0.0,1920.0,1080.0]);
	// // A texture to use with the image
	//let texture = Texture::from_path(Path::new("c18h27no3.jpg"), &TextureSettings::new()).unwrap();
	lazy_static! {	
	    //Create the image object and attach a square Rectangle object inside.
	    static ref image: Image = Image::new().rect([0.0,0.0,1920.0,1080.0]);
	    //A texture to use with the image
	    static ref texture: <GlGraphics as Graphics>::Texture = Texture::from_path(Path::new("IMG_1297.JPG"), &TextureSettings::new()).unwrap();
	}
        let (w, h) = (args.window_size[0], args.window_size[1]);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear([0.0,0.0,0.0,1.0], gl);
	    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
	    //image(&texture,c.transform,gl);
	    <GlGraphics as Graphics>::image(gl, &image, &texture, &DrawState::default(), c.transform);
            // Draw components.
	    line(RED, 3.0, [0.0,h-80.0,w,h-80.0],c.transform,gl);
            for component in self.beatmap.hitobjects.iter() {
		component.draw(time,WINDOW_LENGTH_MS,args,c,gl);
	    }
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }

}
