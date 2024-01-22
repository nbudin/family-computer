use std::{ops::Range, time::Duration};

use super::{APUSequencerMode, NTSC_CPU_FREQUENCY};

#[derive(Clone)]
pub struct APUOscillatorTimer {
  pub prev_sample_count: u64,
  pub sample_count: u64,
  pub timestamp: Duration,
  pub sequencer_mode: APUSequencerMode,
}

impl APUOscillatorTimer {
  pub fn new() -> Self {
    Self {
      prev_sample_count: 0,
      sample_count: 0,
      timestamp: Duration::default(),
      sequencer_mode: APUSequencerMode::FourStep,
    }
  }

  pub fn tick(&mut self, timestamp: Duration) {
    self.prev_sample_count = self.sample_count;
    self.sample_count = self.sample_count.wrapping_add(1);
    self.timestamp = timestamp;
  }

  pub fn current_sample_index(&self, sample_rate: f32) -> f32 {
    (self.sample_count % (sample_rate as u64)) as f32
  }

  pub fn cpu_cycle_range(&self, sample_rate: f32) -> Range<u64> {
    let samples_per_cpu_cycle = NTSC_CPU_FREQUENCY / sample_rate;
    ((self.prev_sample_count as f32) * samples_per_cpu_cycle) as u64
      ..((self.sample_count as f32) * samples_per_cpu_cycle) as u64
  }

  pub fn cycles_per_frame(&self) -> u64 {
    match self.sequencer_mode {
      APUSequencerMode::FourStep => 14917,
      APUSequencerMode::FiveStep => 18642,
    }
  }

  pub fn frame_cycle_range(&self, sample_rate: f32) -> Range<u64> {
    let cycles_per_frame = self.cycles_per_frame();
    let cpu_cycle_range = self.cpu_cycle_range(sample_rate);
    (cpu_cycle_range.start % cycles_per_frame)..(cpu_cycle_range.end % cycles_per_frame)
  }

  pub fn is_half_frame(&self, sample_rate: f32) -> bool {
    let cycle_range = self.frame_cycle_range(sample_rate);

    cycle_range.contains(&7457) || cycle_range.contains(&(self.cycles_per_frame() - 1))
  }

  pub fn is_quarter_frame(&self, sample_rate: f32) -> bool {
    let cycle_range = self.frame_cycle_range(sample_rate);

    cycle_range.contains(&3729)
      || cycle_range.contains(&7457)
      || cycle_range.contains(&11186)
      || cycle_range.contains(&(self.cycles_per_frame() - 1))
  }
}
