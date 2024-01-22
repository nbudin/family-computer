use std::time::Duration;

use super::{APUSequencerMode, NTSC_CPU_FREQUENCY};

#[derive(Clone)]
pub struct APUOscillatorTimer {
  pub sample_count: u64,
  pub timestamp: Duration,
  pub sequencer_mode: APUSequencerMode,
}

impl APUOscillatorTimer {
  pub fn new() -> Self {
    Self {
      sample_count: 0,
      timestamp: Duration::default(),
      sequencer_mode: APUSequencerMode::FourStep,
    }
  }

  pub fn tick(&mut self, timestamp: Duration) {
    self.sample_count = self.sample_count.wrapping_add(1);
    self.timestamp = timestamp;
  }

  pub fn current_sample_index(&self, sample_rate: f32) -> f32 {
    (self.sample_count % (sample_rate as u64)) as f32
  }

  pub fn cpu_cycle_count(&self, sample_rate: f32) -> u64 {
    let samples_per_cpu_cycle = NTSC_CPU_FREQUENCY / sample_rate;
    ((self.sample_count as f32) * samples_per_cpu_cycle) as u64
  }

  pub fn cycles_per_frame(&self) -> u64 {
    match self.sequencer_mode {
      APUSequencerMode::FourStep => 14917,
      APUSequencerMode::FiveStep => 18642,
    }
  }

  pub fn frame_cycle_count(&self, sample_rate: f32) -> u64 {
    self.cpu_cycle_count(sample_rate) / self.cycles_per_frame()
  }

  pub fn is_half_frame(&self, sample_rate: f32) -> bool {
    let cycle_count = self.frame_cycle_count(sample_rate);

    cycle_count == 7457 || cycle_count == self.cycles_per_frame() - 1
  }

  pub fn is_quarter_frame(&self, sample_rate: f32) -> bool {
    let cycle_count = self.frame_cycle_count(sample_rate);

    cycle_count == 3729
      || cycle_count == 7457
      || cycle_count == 11186
      || cycle_count == self.cycles_per_frame() - 1
  }
}
