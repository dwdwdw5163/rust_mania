#[allow(non_snake_case)]
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
use piston::PressEvent;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use anyhow;
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

fn main() -> anyhow::Result<()> {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V4_5;

    // Create a Glutin window.
    let mut windowSettings = WindowSettings::new("spinning-square", [1920, 1080]);
    windowSettings.set_fullscreen(true);
    windowSettings.set_samples(4);
    let mut window: Window = windowSettings
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    
    let beatmap = BeatMap::new("Team Grimoire - C18H27NO3 ([Shana Lesus]) [Alexey's 4K BASIC].osu");
    let mut app = App {
	gl: GlGraphics::new(opengl),
	beatmap: beatmap,
    };

    
    let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
    let sound_data = StaticSoundData::from_file(app.beatmap.audio_file_name.clone(), StaticSoundSettings::default())?;
    let track = manager.add_sub_track(TrackBuilder::default())?;
    let clock = manager.add_clock(ClockSpeed::TicksPerSecond(1000.0))?;
    let hitsound_data = StaticSoundData::from_file("normal-hitnormal.ogg", StaticSoundSettings::default().track(&track).volume(0.2))?;
    clock.start()?;
    println!("{:?}", sound_data.duration());
    let mut sound = manager.play(sound_data.clone())?;
    sound.set_volume(0.2,Tween::default());
    // std::thread::spawn(move || {
    // 	loop {
    // 	    println!("{:?}", &clock.time());
    // 	    std::thread::sleep(std::time::Duration::from_secs(1));
    // 	}
    // });
    std::thread::spawn(move || {
	loop {
	    std::thread::sleep(Duration::from_millis(1000));
	    println!("{:?}",1000.0/time_atomic.load(std::sync::atomic::Ordering::Relaxed) as f64);
	}
    });

    let mut timer = Instant::now();
    let mut events = Events::new(EventSettings{bench_mode:false
					       , max_fps:MAX_FPS, ups:1, ..EventSettings::default()});
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
	    let time_now = Instant::now();
	    time_atomic.store(time_now.duration_since(timer).as_millis() as usize, std::sync::atomic::Ordering::SeqCst);
	    timer = Instant::now();
            app.render(&args, clock.time().ticks);
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
