use kira::{
    clock::ClockTime,
    manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    StartTime, ClockSpeed,
};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_instance() -> Result<(), Box::<dyn std::error::Error>> {
	let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
	let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
	let sound_data = StaticSoundData::from_file("lfdy.mp3", StaticSoundSettings::new())?;
	clock.start()?;
	manager.play(sound_data)?;

	Ok(())
    }
}
