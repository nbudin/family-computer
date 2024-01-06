use crate::audio::{
  oscillator::{Oscillator, Waveform},
  stream_setup::StreamSpawner,
  synth::Synth,
};

pub struct APUSynth {
  synth: Synth,
}

impl APUSynth {
  pub fn new() -> Self {
    Self {
      synth: Synth {
        oscillators: vec![
          Oscillator {
            waveform: Waveform::Square,
            current_sample_index: 0.0,
            frequency_hz: 440.0,
            amplitude: 0.0,
            duty_cycle: 0.5,
          },
          Oscillator {
            waveform: Waveform::Square,
            current_sample_index: 0.0,
            frequency_hz: 440.0,
            amplitude: 0.0,
            duty_cycle: 0.5,
          },
        ],
      },
    }
  }
}

impl StreamSpawner for APUSynth {
  type OutputType = <Synth as StreamSpawner>::OutputType;

  fn spawn_stream<SampleType: cpal::SizedSample + cpal::FromSample<f32> + std::ops::AddAssign>(
    &self,
    device: cpal::Device,
    config: &cpal::StreamConfig,
  ) -> Result<Self::OutputType, anyhow::Error> {
    self.synth.spawn_stream::<SampleType>(device, config)
  }
}
