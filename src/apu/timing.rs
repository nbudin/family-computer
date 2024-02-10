use std::time::Duration;

use super::{APUSequencerMode, NTSC_CPU_FREQUENCY};

pub struct APUTimerInstant<T>
where
  T: std::ops::Rem<u64, Output = T> + std::cmp::PartialEq<u64> + Clone,
{
  pub cycle_count: T,
  pub sequencer_mode: APUSequencerMode,
}

impl<T> APUTimerInstant<T>
where
  T: std::ops::Rem<u64, Output = T> + std::cmp::PartialEq<u64> + Clone,
{
  pub fn cycles_per_frame(&self) -> u64 {
    match self.sequencer_mode {
      APUSequencerMode::FourStep => 14917,
      APUSequencerMode::FiveStep => 18642,
    }
  }

  pub fn frame_normalized(&self) -> Self {
    let cycles_per_frame = self.cycles_per_frame();
    APUTimerInstant {
      cycle_count: self.cycle_count.clone() % cycles_per_frame,
      sequencer_mode: self.sequencer_mode.clone(),
    }
  }

  pub fn is_half_frame(&self) -> bool {
    self.cycle_count == 7457 || self.cycle_count == self.cycles_per_frame() - 1
  }

  pub fn is_quarter_frame(&self) -> bool {
    self.cycle_count == 3729
      || self.cycle_count == 7457
      || self.cycle_count == 11186
      || self.cycle_count == (self.cycles_per_frame() - 1)
  }
}

#[derive(Debug, Clone)]
pub struct CycleCountRange {
  pub start: u64,
  pub end: u64,
}

impl std::ops::Rem<u64> for CycleCountRange {
  type Output = CycleCountRange;

  fn rem(self, rhs: u64) -> Self::Output {
    CycleCountRange {
      start: self.start % rhs,
      end: self.end % rhs,
    }
  }
}

impl std::cmp::PartialEq<u64> for CycleCountRange {
  fn eq(&self, other: &u64) -> bool {
    self.start <= *other && *other < self.end
  }
}

#[derive(Debug, Clone)]
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

  pub fn cpu_cycle_range(&self, sample_rate: f32) -> APUTimerInstant<CycleCountRange> {
    let samples_per_cpu_cycle = NTSC_CPU_FREQUENCY / sample_rate;
    APUTimerInstant {
      sequencer_mode: self.sequencer_mode.clone(),
      cycle_count: CycleCountRange {
        start: ((self.prev_sample_count as f32) * samples_per_cpu_cycle) as u64,
        end: ((self.sample_count as f32) * samples_per_cpu_cycle) as u64,
      },
    }
  }

  pub fn frame_cycle_range(&self, sample_rate: f32) -> APUTimerInstant<CycleCountRange> {
    let cpu_cycle_range = self.cpu_cycle_range(sample_rate);
    cpu_cycle_range.frame_normalized()
  }

  pub fn is_half_frame(&self, sample_rate: f32) -> bool {
    let cycle_range = self.frame_cycle_range(sample_rate);
    cycle_range.is_half_frame()
  }

  pub fn is_quarter_frame(&self, sample_rate: f32) -> bool {
    let cycle_range = self.frame_cycle_range(sample_rate);
    cycle_range.is_quarter_frame()
  }
}
