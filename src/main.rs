use cpal::{default_host, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp::{interpolate::linear::Linear, signal, Signal};
use rustfft::num_complex::Complex32;
use std::sync::atomic::Ordering;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
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

use rustfft::{FftPlanner, num_complex::Complex};

const BUFFER_LEN: usize = 1024;
static TIME_MS: AtomicU64 = AtomicU64::new(0);

#[derive(Parser, Debug, Default)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// audio path
   #[clap(short, long, value_parser)]
   path: String,
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
    buffer: Arc<Mutex<[Complex32;BUFFER_LEN]>>,
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

	let buffer = Arc::new(Mutex::new([Complex32{re:0.0,im:0.0};BUFFER_LEN]));
	let buffer_2 = buffer.clone();
	
        let channels = config.channels();

        let err_fn = move |err| {
            log::error!("an error occurred on stream: {}", err);
        };

        fn write_output_data<T>(output: &mut [T], channels: u16, writer: &PlaybackStateHandle, buffer: &Arc<Mutex<[Complex32;BUFFER_LEN]>>)
        where
            T: cpal::Sample,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(state) = guard.as_mut() {
		    let mut buffer_guard = buffer.try_lock();
                    for frame in output.chunks_mut(channels.into()) {
                        for sample in frame.iter_mut() {
			    *sample =
                                cpal::Sample::from(state.samples.get(state.time).unwrap_or(&0f32));
                        }
			if let Ok(ref mut buf_guard) = buffer_guard {
			    if state.time < buf_guard.len() {
				buf_guard[state.time] = Complex32{ re: *state.samples.get(state.time).unwrap_or(&0f32), im:0.0};
			    }
			    AtomicU64::fetch_add(&TIME_MS,1,Ordering::Relaxed);
			}
			//AtomicU64::fetch_add(&TIME_MS,1,Ordering::Relaxed);
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
                move |data, _: &_| write_output_data::<f32>(data, channels, &state, &buffer),
                err_fn,
            )?,
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<i16>(data, channels, &state, &buffer),
                err_fn,
            )?,
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<u16>(data, channels, &state, &buffer),
                err_fn,
            )?,
        };

        stream.play()?;

        Ok(PlayHandle {
            _stream: stream,
            state: state_2,
	    buffer: buffer_2,
        })
    }

}


fn main() -> Result<()> {

    let args = Args::parse();
    let path = args.path;

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

    std::thread::spawn(move ||{
	let mut planner = FftPlanner::new();
	let fft = planner.plan_fft_forward(BUFFER_LEN);
	let buffer = handle.buffer;
	loop {
	    let mut buf = buffer.lock().unwrap();
	    fft.process(&mut *buf);
	    //println!("fft process once");
	    drop(buf);
	    std::thread::sleep(std::time::Duration::from_millis(8));
	}
	
    });

    
    let bar = ProgressBar::new(duration as u64);
    std::thread::spawn(move || {
	loop {
	    //println!("sec: {:?}sec",AtomicU64::load(&TIME_MS, Ordering::Relaxed) as f64/sample_rate);
	    std::thread::sleep(std::time::Duration::from_millis(100));
	    bar.set_position((AtomicU64::load(&TIME_MS,Ordering::Relaxed) as f64/sample_rate as f64) as u64);
	}
    });
    
    done_rx.recv()?;
    println!("done");
    Ok(())
}
