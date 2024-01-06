use crate::audio::{
  oscillator::{Oscillator, Waveform},
  stream_setup::StreamSpawner,
  synth::Synth,
};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub enum APUSynthChannel {
  Pulse1,
  Pulse2,
  Triangle,
}

pub struct APUSynth {
  synth: Synth<APUSynthChannel>,
}

impl APUSynth {
  pub fn new() -> Self {
    Self {
      synth: Synth {
        oscillators: [
          (
            APUSynthChannel::Pulse1,
            Oscillator {
              waveform: Waveform::Square,
              current_sample_index: 0.0,
              frequency_hz: 440.0,
              amplitude: 0.0,
              duty_cycle: 0.5,
            },
          ),
          (
            APUSynthChannel::Pulse2,
            Oscillator {
              waveform: Waveform::Square,
              current_sample_index: 0.0,
              frequency_hz: 440.0,
              amplitude: 0.0,
              duty_cycle: 0.5,
            },
          ),
          (
            APUSynthChannel::Triangle,
            Oscillator {
              waveform: Waveform::Triangle,
              current_sample_index: 0.0,
              frequency_hz: 440.0,
              amplitude: 0.0,
              duty_cycle: 0.5,
            },
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
