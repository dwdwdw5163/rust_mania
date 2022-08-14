#[allow(non_snake_case)]
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use graphics::{Rectangle, Graphics, image, Context};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use graphics::{Image, clear, draw_state::DrawState, rectangle, line};
use graphics::rectangle::square;
use core::f64;
use std::path::Path;
use lazy_static::lazy_static;

use gfx::{traits::*, Encoder};
use gfx::format::{DepthStencil, Formatted, Srgba8};
use gfx::memory::Typed;
use gfx_graphics::{Flip, Gfx2d, GfxGraphics,};


use crate::beatmap::{BeatMap, HitObject};




pub struct App {
    //pub gl: GlGraphics, // OpenGL drawing backend.
        
    pub beatmap: BeatMap,
}

impl App {
    pub fn render<G: Graphics>(&mut self,c: Context, g: &mut G, args: &RenderArgs, time: u64) {


	    
        
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt;
    }

}
