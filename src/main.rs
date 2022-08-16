#[allow(non_snake_case)]
mod app;
use crate::app::App;
mod beatmap;
use crate::beatmap::{BeatMap,Renderable};

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;


use gfx::handle::Texture;
use gfx_graphics::TextureSettings;
use glutin_window::GlutinWindow as Window;
use graphics::Graphics;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{PressEvent, OpenGLWindow};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use anyhow;
use piston_window::GfxFactory;
use std::sync::atomic::AtomicUsize;
use std::time::{Instant, Duration};

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

    let mut gl =  GlGraphics::new(opengl);
    
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
    sound.set_volume(0.2,Tween::default())?;

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

    let bg = <GlGraphics as Graphics>::Texture::from_path("IMG_1297.JPG", &TextureSettings::new()).unwrap();
    //event Loop
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
	    let time_now = Instant::now();
	    time_atomic.store(time_now.duration_since(timer).as_millis() as usize, std::sync::atomic::Ordering::SeqCst);
	    timer = Instant::now();
	    gl.draw(args.viewport(), |c, gl| {
		use graphics::*;
		clear([0.0, 0.0, 0.0, 1.0], gl);			
		Image::new().rect([0.0,0.0,1920.0,1080.0]).draw(&bg, &c.draw_state, c.transform, gl);
		let (w, h) = (args.window_size[0], args.window_size[1]);
		
		const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
		
		// Draw components.
		line(RED, 3.0, [0.0,h-80.0,w,h-80.0],c.transform, gl);
		for component in beatmap.hitobjects.iter() {
		    component.draw(clock.time().ticks, WINDOW_LENGTH_MS, &args,c, gl);		    
		}
	    });
	    
	}
	    

	
        if let Some(args) = e.update_args() {
            //app.update(&args);
	    if clock.time().ticks > sound_data.duration().as_millis() as u64 {
		break;
	    }
	    println!("time: {:?}",clock.time().ticks);
        }

	if let Some(args) = e.press_args() {
	    // use piston::Input::Button;
	    // if let Button::Keyboard(Key::A) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    // if let Button::Keyboard(Key::S) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    // if let Button::Keyboard(Key::D) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    // if let Button::Keyboard(Key::Space) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    // if let Button::Keyboard(Key::L) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    // if let Button::Keyboard(Key::Semicolon) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    // if let Button::Keyboard(Key::Quote) = args {
	    // 	manager.play(hitsound_data.clone())?;
	    // }
	    manager.play(hitsound_data.clone())?;
	}
    }

    
    //std::thread::sleep(sound_data.duration());
    Ok(())
}
