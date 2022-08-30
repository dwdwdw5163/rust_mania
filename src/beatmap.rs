use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use gfx_device_gl::CommandBuffer;
use lazy_static::lazy_static;
use regex::Regex;
use core::fmt::Debug;
use graphics::*;
use piston::input::{RenderArgs};
use opengl_graphics::GlGraphics;

#[derive(Debug)]
pub enum HitObject {
    Note(Note),
    LongNote(LongNote),
}


pub trait Renderable: Debug {
    fn draw<G: Graphics>(&self, time: u64, window_length_ms: u64, args: &RenderArgs,c: &Context, g: &mut G);
}

impl Renderable for HitObject {
    fn draw<G: Graphics>(&self, time: u64, window_length_ms: u64, args: &RenderArgs,c: &Context, g: &mut G) {
	match self {
	    HitObject::Note(note) => note.draw(time,window_length_ms,args,c,g),
	    HitObject::LongNote(longnote) => longnote.draw(time,window_length_ms,args,c,g),
	}
    }
}

#[derive(Default,Debug)]
pub struct Note {
    pub x: u32,
    y: u32,
    pub time_ms: u64,
    pub typ: u8,
    hitsound: u8,
    pub endtime: u64,
    hitsample: String,
}

#[derive(Default,Debug)]
pub struct LongNote {
    pub x: u32,
    y: u32,
    pub time_ms: u64,
    pub typ: u8,
    hitsound: u8,
    pub endtime: u64,
    hitsample: String,
}

impl Renderable for Note {
    fn draw<G: Graphics>(&self, time: u64, window_length_ms: u64, args: &RenderArgs, c: &Context, g: &mut G) {
	const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
	let win_h: u64 = time + window_length_ms;
	if (win_h > self.time_ms) && (time < self.time_ms) {
	    let h = args.window_size[1]-80.0;
	    let y: f64 = h-((self.time_ms as u64-time)as f64)/(window_length_ms as f64)*h;
	    rectangle(GREEN, [self.x as f64+300., y,-64.0,-32.0], c.transform, g);
	    
	}
    }
}
impl Renderable for LongNote {
    fn draw<G: Graphics>(&self, time: u64, window_length_ms: u64, args: &RenderArgs, c: &Context, g: &mut G) {
	const PINK: [f32; 4] = [0.967, 0.01, 0.58, 0.8];
	let win_h: u64 = time + window_length_ms;
	if (self.typ == 128) && (win_h > self.time_ms) && (time < self.endtime) {
	    let h = args.window_size[1]-80.0;
	    let head: f64 = self.endtime as f64 - win_h as f64;
	    let tail: f64 = self.time_ms as f64- time as f64;
	    let y_top: f64 = if head >= 0.0 {0.0} else {-head/(window_length_ms as f64)*h};
	    let y_height: f64 = if tail <= 0.0 {h-y_top} else {(h-tail/(window_length_ms as f64)*h) - y_top};
	    rectangle(PINK, [self.x as f64+300., y_top, 64.0, y_height], c.transform, g);
	    //Some([self.x as f64, y_top, 64.0, y_height])
	}
    }
}

#[derive(Default,Debug)]
pub struct BeatMap {
    //[General]
    pub audio_file_name: String,
    mode: u16,
    //[Metadata]
    title: String,
    artist: String,
    //[Difficulty]
    hpDrainRate: u16,
    circleSize: u16,
    overallDifficulty: u16,//keys
    approachRate: u16,
    silderTickRate: u16,
    //[Events]
    background: String,
    //[Timingpoints]
    //time,beatLength,meter,sampleSet,
    //sampleIndex,volume,uninherited,effects
    //reserved

    //[Hitobjects]
    //https://osu.ppy.sh/wiki/en/Client/File_formats/Osu_%28file_format%29
    //x,y,time,type,hitSound,objectParams,hitSample3
    pub hitobjects: Vec<HitObject>,
}

impl BeatMap {
    pub fn new<P>(filename: P) -> BeatMap
    where P: AsRef<Path>, {
	let mut state = 0;
	let mut beatmap = BeatMap::default();
	lazy_static! {
            static ref re_keyValue: Regex = Regex::new("(.*):( *)(.*)").unwrap();
	}
	
	if let Ok(lines) = read_lines(filename) {
	    for line in lines {
		if let Ok(s) = line {
		    let temp = re_keyValue.captures(&s);
		    if let Some(t) = temp {
			//println!("{:?}, {:?}",&t[1],&t[3]);
			match &t[1] {
			    "AudioFilename" => beatmap.audio_file_name = t[3].to_string(),
			    "Mode" => beatmap.mode = t[3].to_string().parse().unwrap(),
			    "Title" => beatmap.title = t[3].to_string(),
			    "Artist" => beatmap.artist = t[3].to_string(),
			    "HPDrainRate" => beatmap.hpDrainRate = t[3].to_string().parse().unwrap(),
			    "CircleeSize" => beatmap.circleSize = t[3].to_string().parse().unwrap(),
			    "OverallDifficulty" => beatmap.overallDifficulty = t[3].to_string().parse().unwrap(),
			    "ApproachRate" => beatmap.approachRate = t[3].to_string().parse().unwrap(),
			    "SliderTickRate" => beatmap.silderTickRate = t[3].to_string().parse().unwrap(),
		  	    _ => {},	
			}
			if &t[3] == "" {
			    let spilt = s.split(",");
			    if let (v,Some(_)) = spilt.size_hint() {
				if v < 2 {
				    panic!("Error while reading HitObject");
				}
			    }
			    let vec: Vec<&str> = spilt.collect();
			    if vec[3] == "1" {
				let mut hitobj = Note::default();
				hitobj.x = vec[0].parse().unwrap();
				hitobj.y = vec[1].parse().unwrap();
				hitobj.time_ms = vec[2].parse().unwrap();
				hitobj.typ = vec[3].parse().unwrap();
				hitobj.hitsound = vec[4].parse().unwrap();
				let vec1: Vec<&str> = vec[5].split(":").collect();
				//println!("{:?}",vec1[0]);
				hitobj.endtime = vec1[0].parse().unwrap();
				beatmap.hitobjects.push(HitObject::Note(hitobj));
			    }
			    if vec[3] == "128" {
				let mut hitobj = LongNote::default();
				hitobj.x = vec[0].parse().unwrap();
				hitobj.y = vec[1].parse().unwrap();
				hitobj.time_ms = vec[2].parse().unwrap();
				hitobj.typ = vec[3].parse().unwrap();
				hitobj.hitsound = vec[4].parse().unwrap();
				let vec1: Vec<&str> = vec[5].split(":").collect();
				//println!("{:?}",vec1[0]);
				hitobj.endtime = vec1[0].parse().unwrap();
				beatmap.hitobjects.push(HitObject::LongNote(hitobj));
			    }
			  
			}
		    }
		}
	    }
	}
	beatmap
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
