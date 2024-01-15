use std::f32::consts::PI;

use fastapprox::fast::sinfull;

use crate::audio::audio_channel::AudioChannel;

use super::{
  linear_counter::APULinearCounter,
  registers::{APUTimerRegister, APUTriangleControlRegister},
  APULengthCounter, APUSequencer,
};

const TWO_PI: f32 = PI * 2.0;

#[derive(Debug)]
pub enum APUTriangleOscillatorCommand {
  SetFrequency(f32),
  SetEnabled(bool),
}

#[derive(Clone)]
pub struct APUTriangleOscillator {
  current_sample_index: f32,
  frequency: f32,
  harmonics: usize,
  enabled: bool,
}

impl APUTriangleOscillator {
  pub fn new() -> Self {
    Self {
      current_sample_index: 0.0,
      frequency: 440.0,
      harmonics: 20,
      enabled: false,
    }
  }
}

impl AudioChannel for APUTriangleOscillator {
  fn get_next_sample(&mut self, sample_rate: f32) -> f32 {
    self.current_sample_index = (self.current_sample_index + 1.0) % sample_rate;

    if !self.enabled {
      return 0.0;
    }

    let mut output: f32 = 0.0;

    for i in 0..self.harmonics {
      let n = ((i * 2) + 1) as f32;
      let sample_index_radians =
        (n * self.frequency * TWO_PI * self.current_sample_index) / sample_rate;
      output += -sinfull(sample_index_radians) / n;
    }

    (2.0 / PI) * output
  }

  fn handle_command(&mut self, command: Box<dyn std::any::Any + Send + Sync>) {
    let Ok(command) = command.downcast::<APUTriangleOscillatorCommand>() else {
      return;
    };

    match command.as_ref() {
      APUTriangleOscillatorCommand::SetFrequency(frequency) => self.frequency = *frequency,
      APUTriangleOscillatorCommand::SetEnabled(enabled) => self.enabled = *enabled,
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUTriangleChannel {
  pub control: APUTriangleControlRegister,
  pub timer: APUTimerRegister,
  pub enabled: bool,
  pub length_counter: APULengthCounter,
  pub linear_counter: APULinearCounter,
  pub sequencer: APUSequencer,
}

impl Default for APUTriangleChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl APUTriangleChannel {
  pub fn new() -> Self {
    Self {
      control: 0.into(),
      timer: 0.into(),
      enabled: false,
      linear_counter: APULinearCounter::new(),
      length_counter: APULengthCounter::new(),
      sequencer: APUSequencer::new(),
    }
  }

  pub fn producing_sound(&self) -> bool {
    self.enabled && self.length_counter.counter > 0 && self.linear_counter.counter > 0
  }

  pub fn write_control(&mut self, value: APUTriangleControlRegister) {
    self.linear_counter.counter = value.counter_reload_value();
    self.linear_counter.control_flag = value.control_flag();
    self.length_counter.halt = value.control_flag();

    self.control = value;
  }

  pub fn write_timer_byte(&mut self, value: u8, high_byte: bool) {
    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | (((value & 0b111) as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    self.timer = new_value;
    self.sequencer.timer = self.timer.timer();

    if high_byte {
      let length_counter_load_index = value >> 3;
      self.length_counter.load_length(length_counter_load_index);
      self.linear_counter.reload_flag = true;
    }
  }
}
