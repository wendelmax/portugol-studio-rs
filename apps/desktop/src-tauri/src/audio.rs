//! Reprodução assíncrona de Sons no mesmo dispositivo para todos os clipes
//! de um programa. A saída só é aberta ao começar a reprodução.

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use portugol_service::Sound;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source, buffer::SamplesBuffer};

#[derive(Default)]
pub struct AudioOutput(Rc<RefCell<Option<MixerDeviceSink>>>);

impl AudioOutput {
    pub fn load(&self, path: &Path) -> Result<Box<dyn Sound>, String> {
        let unavailable = || {
            format!(
                "Não foi possível criar ou abrir uma linha de execução de áudio para {}",
                path.display()
            )
        };
        let file = File::open(path).map_err(|_| unavailable())?;
        let decoder = Decoder::try_from(file).map_err(|_| unavailable())?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let samples: Vec<_> = decoder.collect();
        if samples.is_empty() {
            return Err(unavailable());
        }
        let samples = SamplesBuffer::new(channels, sample_rate, samples);
        let duration = samples.total_duration().unwrap_or_default();
        Ok(Box::new(Clip {
            output: Self(Rc::clone(&self.0)),
            samples,
            duration,
            player: None,
            start: Duration::ZERO,
            gain: 1.0,
            looping: false,
            resume: false,
        }))
    }

    fn player(&self) -> Result<Player, String> {
        let mut device = self.0.borrow_mut();
        if device.is_none() {
            *device = Some(DeviceSinkBuilder::open_default_sink().map_err(|error| {
                format!("Não foi possível abrir o dispositivo de áudio: {error}")
            })?);
        }
        Ok(Player::connect_new(
            device.as_ref().expect("saída aberta").mixer(),
        ))
    }
}

struct Clip {
    output: AudioOutput,
    samples: SamplesBuffer,
    duration: Duration,
    player: Option<Player>,
    start: Duration,
    gain: f32,
    looping: bool,
    resume: bool,
}

impl Clip {
    fn start_at(&mut self, start: Duration, looping: bool) -> Result<(), String> {
        let player = self.output.player()?;
        player.set_volume(self.gain);
        self.player = None;
        if looping {
            player.append(self.samples.clone().repeat_infinite().skip_duration(start));
        } else {
            player.append(self.samples.clone().skip_duration(start));
        }
        self.start = start;
        self.looping = looping;
        self.resume = false;
        self.player = Some(player);
        Ok(())
    }
}

impl Sound for Clip {
    fn play(&mut self, looping: bool) -> Result<(), String> {
        let start = if self.resume {
            self.position()
        } else {
            Duration::ZERO
        };
        self.start_at(start, looping)
    }

    fn pause(&mut self) {
        if let Some(player) = &self.player {
            player.pause();
        }
        self.resume = true;
    }

    fn stop(&mut self) {
        self.start = self.position();
        self.player = None;
        self.resume = false;
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn position(&self) -> Duration {
        let Some(player) = &self.player else {
            return self.start;
        };
        if !self.looping && player.empty() {
            return self.duration;
        }
        let elapsed = self.start.saturating_add(player.get_pos());
        if self.looping && !self.duration.is_zero() {
            Duration::from_secs_f64(elapsed.as_secs_f64() % self.duration.as_secs_f64())
        } else {
            elapsed.min(self.duration)
        }
    }

    fn seek(&mut self, position: Duration) -> Result<(), String> {
        let start = position.min(self.duration);
        if self
            .player
            .as_ref()
            .is_some_and(|player| !player.is_paused() && !player.empty())
        {
            self.start_at(start, self.looping)
        } else {
            self.player = None;
            self.start = start;
            self.resume = true;
            Ok(())
        }
    }

    fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
        if let Some(player) = &self.player {
            player.set_volume(gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_wav_bytes(sample_rate: u32, num_samples: usize) -> Vec<u8> {
        let data_size = (num_samples * 2) as u32;
        let mut wav = Vec::with_capacity(44 + data_size as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // PCM chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        for i in 0..num_samples {
            let sample = ((i as f32 * 0.1).sin() * 8000.0) as i16;
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        wav
    }

    #[test]
    fn decodes_wav_without_opening_a_device() {
        let path = std::env::temp_dir().join(format!(
            "portugol-desktop-audio-test-{}.wav",
            std::process::id()
        ));
        std::fs::write(&path, synthetic_wav_bytes(16_000, 1_600)).unwrap();

        let audio = AudioOutput::default();
        let mut clip = audio.load(&path).unwrap();
        assert!(clip.duration() > Duration::ZERO);
        let position = clip.duration() / 2;
        clip.seek(position).unwrap();
        assert_eq!(clip.position(), position);
        assert!(audio.0.borrow().is_none());

        let _ = std::fs::remove_file(&path);
    }
}
