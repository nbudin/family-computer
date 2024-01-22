use crate::audio::{audio_channel::AudioChannel, stream_setup::StreamSpawner, synth::Synth};

use super::{APUNoiseOscillator, APUPulseOscillator, APUTriangleOscillator};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub enum APUSynthChannel {
  Pulse1,
  Pulse2,
  Triangle,
  Noise,
}

pub struct APUSynth {
  synth: Synth<APUSynthChannel>,
}

impl Default for APUSynth {
  fn default() -> Self {
    Self::new()
  }
}

impl APUSynth {
  pub fn new() -> Self {
    Self {
      synth: Synth {
        channels: [
          (
            APUSynthChannel::Pulse1,
            Box::new(APUPulseOscillator::new()) as Box<dyn AudioChannel>,
          ),
          (
            APUSynthChannel::Pulse2,
            Box::new(APUPulseOscillator::new()) as Box<dyn AudioChannel>,
          ),
          (
            APUSynthChannel::Triangle,
            Box::new(APUTriangleOscillator::new()) as Box<dyn AudioChannel>,
          ),
          (
            APUSynthChannel::Noise,
            Box::new(APUNoiseOscillator::new()) as Box<dyn AudioChannel>,
          ),
        ]
        .into_iter()
        .collect(),
      },
    }
  }
}

impl StreamSpawner for APUSynth {
  type OutputType = <Synth<APUSynthChannel> as StreamSpawner>::OutputType;

  fn spawn_stream<
    SampleType: cpal::SizedSample
      + cpal::FromSample<f32>
      + core::iter::Sum<SampleType>
      + core::ops::Add<SampleType, Output = SampleType>,
  >(
    &self,
    device: cpal::Device,
    config: &cpal::StreamConfig,
  ) -> Result<Self::OutputType, anyhow::Error> {
    self.synth.spawn_stream::<SampleType>(device, config)
  }
}
