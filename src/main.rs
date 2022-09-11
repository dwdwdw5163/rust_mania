use cpal::{default_host, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp::{interpolate::linear::Linear, signal, Signal};
use rustfft::num_complex::Complex32;
use rustfft::num_traits::Pow;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{channel, self};
use std::sync::{Arc, Mutex};
use std::usize;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use color_eyre::eyre::{eyre, Result};

use indicatif::ProgressBar;

use clap::Parser;
use std::sync::atomic::AtomicU64;

use glutin_window::GlutinWindow as Window;
use graphics::Graphics;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::{PressEvent, OpenGLWindow};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use rustfft::{FftPlanner, num_complex::Complex};


mod beatmap;
use crate::beatmap::{BeatMap,Renderable};

const BUFFER_LEN: usize = 4096;
static TIME_MS: AtomicU64 = AtomicU64::new(0);

static tick: AtomicU64 = AtomicU64::new(0);

#[derive(Parser, Debug, Default)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// audio path
    #[clap(short, long, value_parser)]
    audio_path: String,
    /// .osu path
    #[clap(short, long, value_parser)]
    osu_path: String,
}


struct PlaybackState {
    time: usize,
    samples: Vec<f32>,
    changed_cbs: Vec<Box<dyn Fn() + Send>>,
    changed_cbs_triggered_at: usize,
    done_cbs: Vec<Box<dyn Fn() + Send>>,
    sample_rate: usize,
}

type PlaybackStateHandle = Arc<Mutex<Option<PlaybackState>>>;


pub struct PlayHandle {
    _stream: Stream,
    state: PlaybackStateHandle,
}

impl PlayHandle {
    pub fn connect_changed<F: Fn() + 'static + Send>(&self, f: F) {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();
        state.changed_cbs.push(Box::new(f));
    }

    pub fn connect_done<F: Fn() + 'static + Send>(&self, f: F) {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();

        if state.time >= state.samples.len() {
            f();
        } else {
            state.done_cbs.push(Box::new(f));
        }
    }

}

#[derive(Clone)]
pub struct AudioClip {
    pub name: String,
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}


impl AudioClip {
    pub fn resample(&self, sample_rate: u32) -> AudioClip {
        if self.sample_rate == sample_rate {
            return self.clone();
        }

        let mut signal = signal::from_iter(self.samples.iter().copied());
        let a = signal.next();
        let b = signal.next();

        let linear = Linear::new(a, b);

        AudioClip {
            name: self.name.clone(),
            samples: signal
                .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
                .take(self.samples.len() * (sample_rate as usize) / (self.sample_rate as usize))
                .collect(),
            sample_rate,
	}
    }

    pub fn import(name: String, path: String) -> Result<AudioClip> {
        // Create a media source. Note that the MediaSource trait is automatically implemented for File,
        // among other types.
        let file = Box::new(File::open(Path::new(&path))?);

        //let creation_time = file.metadata()?.created()?;

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(file, Default::default());

        // Create a hint to help the format registry guess what format reader is appropriate. In this
        // example we'll leave it empty.
        let hint = Hint::new();

        // Use the default options when reading and decoding.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // Probe the media source stream for a format.
        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        // Get the format reader yielded by the probe operation.
        let mut format = probed.format;

        // Get the default track.
        let track = format
            .default_track()
            .ok_or_else(|| eyre!("No default track"))?;

        // Create a decoder for the track.
        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts)?;

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        let mut sample_count = 0;
        let mut sample_buf = None;
        let channels = track
            .codec_params
            .channels
            .ok_or_else(|| eyre!("Unknown channel count"))?;

        let mut clip = AudioClip {
            name,
            samples: Vec::new(),
            sample_rate: track
                .codec_params
                .sample_rate
                .ok_or_else(|| eyre!("Unknown sample rate"))?,
	};

        loop {
            // Get the next packet from the format reader.
            let packet = match format.next_packet() {
                Ok(packet_ok) => packet_ok,
                Err(Error::IoError(ref packet_err))
                    if packet_err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(packet_err) => {
                    return Err(packet_err.into());
                }
            };

            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // The decoded audio samples may now be accessed via the audio buffer if per-channel
                    // slices of samples in their native decoded format is desired. Use-cases where
                    // the samples need to be accessed in an interleaved order or converted into
                    // another sample format, or a byte buffer is required, are covered by copying the
                    // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                    // example below, we will copy the audio buffer into a sample buffer in an
                    // interleaved order while also converting to a f32 sample format.

                    // If this is the *first* decoded packet, create a sample buffer matching the
                    // decoded audio buffer format.
                    if sample_buf.is_none() {
                        // Get the audio buffer specification.
                        let spec = *audio_buf.spec();

                        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                        let duration = audio_buf.capacity() as u64;

                        // Create the f32 sample buffer.
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);
                        let mono: Vec<f32> = buf
                            .samples()
                            .iter()
                            .step_by(channels.count())
                            .copied()
                            .collect();
                        clip.samples.extend_from_slice(&mono);

                        // The samples may now be access via the `samples()` function.
                        sample_count += buf.samples().len();
                        log::info!("\rDecoded {} samples", sample_count);
                    }
                }
                Err(Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }

        Ok(clip)
    }

    pub fn play(&self) -> Result<PlayHandle> {
        let host = default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| eyre!("No output device"))?;
        log::info!("Output device: {}", device.name()?);
        let config = device.default_output_config()?;

        log::info!("Begin playback...");

        let sample_rate = config.sample_rate().0;

	let state = PlaybackState {
            time: 0,
            samples: self.resample(sample_rate).samples,
	    done_cbs: vec![],
            changed_cbs: vec![],
            changed_cbs_triggered_at: 0,
            sample_rate: sample_rate as usize,
        };
        let state: PlaybackStateHandle = Arc::new(Mutex::new(Some(state)));
        let state_2 = state.clone();

	
        let channels = config.channels();

        let err_fn = move |err| {
            log::error!("an error occurred on stream: {}", err);
        };

        fn write_output_data<T>(output: &mut [T], channels: u16, writer: &PlaybackStateHandle)
        where
            T: cpal::Sample,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(state) = guard.as_mut() {
		    
                    for frame in output.chunks_mut(channels.into()) {
                        for sample in frame.iter_mut() {
			    *sample =
                                cpal::Sample::from(state.samples.get(state.time).unwrap_or(&0f32));
                        }
			AtomicU64::fetch_add(&TIME_MS,1,Ordering::Relaxed);
                        state.time += 1;
                    }
                    if state.time >= state.samples.len() {
                        for cb in &*state.done_cbs {
                            cb();
                        }
                    }
                    if state.time >= state.changed_cbs_triggered_at + state.sample_rate / 100 {
                        for cb in &*state.changed_cbs {
                            cb();
                        }
                        state.changed_cbs_triggered_at = state.time;
                    }
                }
            }
        }

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<f32>(data, channels, &state),
                err_fn,
            )?,
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<i16>(data, channels, &state),
                err_fn,
            )?,
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<u16>(data, channels, &state),
                err_fn,
            )?,
        };

        stream.play()?;

        Ok(PlayHandle {
            _stream: stream,
            state: state_2,
	})
    }

}

fn hann_window<const N: usize>(buffer: &mut [Complex32;N]) {
    for (i, v) in buffer.iter_mut().enumerate() {
	v.re = v.re*0.5*(1.0 - f32::cos(2.0*i as f32/(N-1) as f32 * std::f32::consts::PI));
    }
}

fn main() -> Result<()> {

    let args = Args::parse();
    let path = args.audio_path;

    //fftplanner
    
    
    println!("begin import");
    //let clip: AudioClip =
	//AudioClip::import(String::from("test"), String::from("/home/zhang/Downloads/YOUTH - Live Fast, Die Young.mp3"))?;
    let clip: AudioClip =
	AudioClip::import(String::from("test"), path)?;
    
    let sample_rate = clip.sample_rate as f64;
    let duration = clip.samples.len() as f64/(clip.sample_rate as f64);
    println!("finished import and play");
    let handle = clip.play()?;
    let (done_tx,done_rx) = channel::<()>();
    handle.connect_done(move || {
	done_tx.send(()).unwrap();
    });

    let mut bar_bin: Vec<ProgressBar> = Vec::new();
    for _ in 0..16 {
	bar_bin.push(ProgressBar::new(500));
    }

    let (tx,rx) = mpsc::channel::<[u64;BUFFER_LEN/2]>();
    
    std::thread::spawn(move ||{
	let mut planner = FftPlanner::new();
	let fft = planner.plan_fft_forward(BUFFER_LEN);
	let mut timer = std::time::Instant::now();
	let handle_guard = handle.state.lock().unwrap();
	let samples = handle_guard.as_ref().unwrap().samples.clone();
	drop(handle_guard);
	let mut buf = [Complex32{re:0.0,im:0.0};BUFFER_LEN];
	loop {
	    let mut fft_amp = [0;BUFFER_LEN/2];
	    let time_now = std::time::Instant::now();
	    let time = time_now.duration_since(timer);
	    tick.store(time.as_millis() as u64, Ordering::Relaxed);
	    //timer = time_now;
	    //println!("time {:?}",time.as_secs_f32());

	    for (i,v) in buf.iter_mut().enumerate() {
		if let Some(val) = samples.get((i as f64+time.as_secs_f64() * sample_rate) as usize) {
		    *v = Complex32{re:*val,im:0.0};
		    //println!("{:?}",val);
		}
	    }
	    //println!("{:?}",&buf[0..16]);
	    hann_window(&mut buf);
	    //println!("{:?}",&buf[0..16]);
	    fft.process(&mut buf);
	    //println!("fft process once");
	    
	    for i in 0..BUFFER_LEN/2 {
		fft_amp[i] = buf[i].norm_sqr() as u64;
	    }
	    let mut max = fft_amp.iter().max().unwrap();
	    if *max == 0 {
		max = &1;
	    }
	    fft_amp = fft_amp.map(|x| x*800/max.deref());
	    drop(buf);

	    //println!("{:?}", fft_amp);
	    tx.send(fft_amp).unwrap();
	    

	    std::thread::sleep(std::time::Duration::from_millis(4));
	}
	
    });

    
    
    std::thread::spawn(move || {
	loop {
	    //println!("sec: {:?}sec",AtomicU64::load(&TIME_MS, Ordering::Relaxed) as f64/sample_rate);
	    std::thread::sleep(std::time::Duration::from_millis(16));
	    //bar.set_position((AtomicU64::load(&TIME_MS,Ordering::Relaxed) as f64/sample_rate as f64) as u64);
	}
    });

    
    let opengl = OpenGL::V3_2;

    let beatmap = BeatMap::new(args.osu_path);

    let mut windowSettings = WindowSettings::new("fft", [1920,1080]);
    let mut window: Window = windowSettings
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);

    let mut events = Events::new(EventSettings{max_fps:144, ups:0,..EventSettings::default()});
    
    //Create the image object and attach a square Rectangle object inside.
    use graphics::*;
    let image   = Image::new().rect([0.0,0.0,1920.0,1080.0]);
    //A texture to use with the image
    let texture = Texture::from_path(Path::new("/home/zhang/Pictures/Wallpapers/crop.png"), &TextureSettings::new()).unwrap();
    
    while let Some(e) = events.next(&mut window) {
	if let  Ok(_) = done_rx.try_recv() {
	    break;
	}
	if let Some(args) = e.render_args() {
	    gl.draw(args.viewport(), |c, gl| {
		use graphics::*;
		clear([0.0,0.0,0.0,1.0], gl);

		//image(&texture, [[1.0/3263.0*2.0,0.0,-1.0],[0.0,-1.0/1835.0*2.0,1.0]], gl);
				
		let (w, h) = (args.window_size[0], args.window_size[1]);
		
		const COLOR: [f32; 4] = [0.0, 0.8, 0.0, 1.0];
		let buffer = rx.recv().unwrap();

		for (i,val) in buffer.iter().enumerate() {
		    rectangle(COLOR,[w/buffer.len() as f64*i as f64*20.0,3.0*h/4.0,w/buffer.len() as f64*10.0,(*val as f64+1.0).sqrt()*-8.0],c.transform,gl);
		    
		}
		let length = buffer.len();
		for i in 0..(length/20) as usize {
		    line([1.0,0.0,0.0,1.0], 2.0, [w/length as f64*i as f64*20.0, (buffer[i] as f64+1.0).sqrt()*-8.0 + 3.0*h/4.0, w/length as f64*(i+1) as f64*20.0, (buffer[i+1] as f64+1.0).sqrt()*-8.0+h*3.0/4.0], c.transform, gl);
		    
		}

		for component in beatmap.hitobjects.iter() {
		    component.draw(tick.load(Ordering::Relaxed), 500, &args, &c, gl);		    
		}
		line([1.0,1.0,1.0,1.0], 3.0, [0.0,h-80.0,w,h-80.0],c.transform, gl);
		
		rectangle(COLOR,[64.0+300.0,h,64.0,-80.0],c.transform,gl);
		rectangle(COLOR,[192.0+300.0,h,64.0,-80.0],c.transform,gl);
		rectangle(COLOR,[320.0+300.0,h,64.0,-80.0],c.transform,gl);
		rectangle(COLOR,[448.0+300.0,h,64.0,-80.0],c.transform,gl);
		
		
	    });
	    
	}
    }
    
    
    //done_rx.recv()?;
    println!("done");
    Ok(())
}
