use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

struct HitObject {
    x: u32,
    y: u32,
    time: usize,
    typ: u8,
    hitsound: u8,
    objparams: String,
    hitsample: String,
}

pub struct BeatMap {
    //[General]
    audio_file_name: String,
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
    hitobjects: Vec<Hitobjects>,
}

impl BeatMap {
    pub fn new<P>(filename: P) -> BeatMap
    where P: AsRef<Path>, {
	let mut state = 0;
	if let Ok(lines) = read_lines(filename) {
	    for line in lines {
		
	    }
	}
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
