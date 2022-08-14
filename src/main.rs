#[allow(non_snake_case)]
mod app;
use crate::app::App;
mod beatmap;
use crate::beatmap::{BeatMap,Renderable};

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate gfx_graphics;
extern crate gfx;
extern crate gfx_device_gl;
extern crate piston_window;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{PressEvent, OpenGLWindow};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use anyhow;
use piston_window::GfxFactory;
use std::sync::atomic::AtomicUsize;
use std::time::{Instant, Duration};

use gfx::{traits::*, Encoder};
use gfx::format::{DepthStencil, Formatted, Srgba8};
use gfx::memory::Typed;
use gfx_graphics::{Flip, Gfx2d,};
use piston_window::*;

use kira::{
    clock::ClockTime,
    manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    StartTime, ClockSpeed,
    tween::Tween,
    track::TrackBuilder,
};


static time_atomic: std::sync::atomic::AtomicUsize = AtomicUsize::new(0);

static MAX_FPS: u64 = 144;

static WIDTH: f64 = 1920.0;
static HEIGHT: f64 = 1080.0;
static WINDOW_LENGTH_MS: u64 = 512;

fn main() -> anyhow::Result<()> {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let samples = 4;
    // Create a Glutin window.
    let mut windowSettings = WindowSettings::new("spinning-square", [WIDTH, HEIGHT]);
    windowSettings.set_fullscreen(true);
    windowSettings.set_samples(samples);
    let mut window: Window = windowSettings
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let (mut device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s) as *const std::os::raw::c_void);
    // Create the main color/depth targets.
    let aa = samples as gfx::texture::NumSamples;
    let dim = (WIDTH as u16, HEIGHT as u16, 1, aa.into());
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
    
    
    let beatmap = BeatMap::new("Team Grimoire - C18H27NO3 ([Shana Lesus]) [Alexey's 4K BASIC].osu");
    // let mut app = App {
    // 	gl: GlGraphics::new(opengl),
    // 	beatmap: beatmap,
    // };

    //kira sound settings
    let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
    let sound_data = StaticSoundData::from_file(beatmap.audio_file_name.clone(), StaticSoundSettings::default())?;
    let track = manager.add_sub_track(TrackBuilder::default())?;
    let clock = manager.add_clock(ClockSpeed::TicksPerSecond(1000.0))?;
    let hitsound_data = StaticSoundData::from_file("normal-hitnormal.ogg", StaticSoundSettings::default().track(&track).volume(0.2))?;
    clock.start()?;
    println!("{:?}", sound_data.duration());
    let mut sound = manager.play(sound_data.clone())?;
    sound.set_volume(0.2,Tween::default());

    //print debug
    std::thread::spawn(move || {
	loop {
	    std::thread::sleep(Duration::from_millis(1000));
	    println!("{:?}",1000.0/time_atomic.load(std::sync::atomic::Ordering::Relaxed) as f64);
	}
    });

    //fps counter
    let mut timer = Instant::now();
    let mut events = Events::new(EventSettings{bench_mode:false, max_fps:MAX_FPS, ups:1, ..EventSettings::default()});

    //event Loop
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
	    let time_now = Instant::now();
	    time_atomic.store(time_now.duration_since(timer).as_millis() as usize, std::sync::atomic::Ordering::SeqCst);
	    timer = Instant::now();
	    g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
		clear([0.8, 0.8, 0.8, 1.0], g);

		let (w, h) = (args.window_size[0], args.window_size[1]);
		
		const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
		
		// Draw components.
		line(RED, 3.0, [0.0,h-80.0,w,h-80.0],c.transform,g);
		for component in beatmap.hitobjects.iter() {
		    component.draw(clock.time().ticks, WINDOW_LENGTH_MS, &args,c, g);		    
		}
	    });
	    encoder.flush(&mut device);
	}
	    
            
        if let Some(_) = e.after_render_args() {
	    device.cleanup();
	} 
	
        if let Some(args) = e.update_args() {
            //app.update(&args);
	    if clock.time().ticks > sound_data.duration().as_millis() as u64 {
		break;
	    }
	    println!("time: {:?}",clock.time().ticks);
        }

	if let Some(args) = e.press_args() {
	    let mut hit_sound = manager.play(hitsound_data.clone()).unwrap();
	}
    }

    
    //std::thread::sleep(sound_data.duration());
    Ok(())
}
