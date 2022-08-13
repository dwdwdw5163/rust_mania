use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use lazy_static::lazy_static;
use regex::Regex;
use anyhow;


#[derive(Default,Debug)]
pub struct HitObject {
    pub x: u32,
    y: u32,
    pub time_ms: u64,
    pub typ: u8,
    hitsound: u8,
    pub endtime: u64,
    hitsample: String,
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
			    let mut hitobj = HitObject::default();
			    hitobj.x = vec[0].parse().unwrap();
			    hitobj.y = vec[1].parse().unwrap();
			    hitobj.time_ms = vec[2].parse().unwrap();
			    hitobj.typ = vec[3].parse().unwrap();
			    hitobj.hitsound = vec[4].parse().unwrap();
			    if hitobj.typ == 128 {
				let vec1: Vec<&str> = vec[5].split(":").collect();
				//println!("{:?}",vec1[0]);
				hitobj.endtime = vec1[0].parse().unwrap();
			    }

			    beatmap.hitobjects.push(hitobj);
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
