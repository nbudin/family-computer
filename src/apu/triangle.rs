use std::{f32::consts::PI, time::Duration};

use fastapprox::fast::sinfull;
use tinyvec::array_vec;

use crate::{apu::COMMAND_BUFFER_SIZE, audio::audio_channel::AudioChannel};

use super::{
  linear_counter::APULinearCounter,
  registers::{APUTimerRegister, APUTriangleControlRegister},
  timing::APUOscillatorTimer,
  APUChannelStateTrait, APULengthCounter, APUSequencer, APUSequencerMode, CommandBuffer,
};

const TWO_PI: f32 = PI * 2.0;

#[derive(Debug, PartialEq, Eq, Hash, Default)]
pub enum APUTriangleOscillatorCommand {
  #[default]
  NoOp,
  WriteControl(APUTriangleControlRegister),
  WriteTimerRegister(APUTimerRegister),
  SetEnabled(bool),
  LoadLengthCounterByIndex(u8),
  SetAPUSequencerMode(APUSequencerMode),
}

#[derive(Clone)]
pub struct APUTriangleOscillator {
  harmonics: usize,
  enabled: bool,
  length_counter: APULengthCounter,
  linear_counter: APULinearCounter,
  sequencer: APUSequencer,
  timer: APUOscillatorTimer,
  timer_register: APUTimerRegister,
}

impl APUTriangleOscillator {
  pub fn new() -> Self {
    Self {
      harmonics: 20,
      enabled: false,
      length_counter: APULengthCounter::new(),
      linear_counter: APULinearCounter::new(),
      sequencer: APUSequencer::new(),
      timer: APUOscillatorTimer::new(),
      timer_register: APUTimerRegister::new(),
    }
  }

  fn frequency(&self) -> f32 {
    self.timer_register.triangle_frequency()
  }
}

impl AudioChannel for APUTriangleOscillator {
  fn get_next_sample(&mut self, sample_rate: f32, timestamp: Duration) -> f32 {
    // let prev_cycle_count = self.timer.cpu_cycle_count(sample_rate);
    self.timer.tick(timestamp);

    if !self.enabled {
      return 0.0;
    }

    if self.length_counter.counter == 0 || self.linear_counter.counter == 0 {
      return 0.0;
    }

    // let cycle_count = self.timer.cpu_cycle_count(sample_rate);
    // let elapsed_cycles = cycle_count - prev_cycle_count;

    // for _i in 0..(elapsed_cycles * 2) {
    //   self.sequencer.tick(
    //     self.enabled && self.linear_counter.counter > 0 && self.length_counter.counter > 0,
    //     |sequence| {
    //       // this represents a step in the 15 -> 0, 0 -> 15 sequence
    //       (sequence + 1) % 32
    //     },
    //   );
    // }

    if self.timer.is_quarter_frame(sample_rate) {
      self.linear_counter.tick();
    }
    if self.timer.is_half_frame(sample_rate) {
      self.length_counter.tick();
    }

    let mut output: f32 = 0.0;
    let current_sample_index = self.timer.current_sample_index(sample_rate);

    for i in 0..self.harmonics {
      let n = ((i * 2) + 1) as f32;
      let sample_index_radians =
        (n * self.frequency() * TWO_PI * current_sample_index) / sample_rate;
      output += -sinfull(sample_index_radians) / n;
    }

    (2.0 / PI) * output
  }

  fn handle_command(&mut self, command: Box<dyn std::any::Any + Send + Sync>) {
    let Ok(command) = command.downcast::<APUTriangleOscillatorCommand>() else {
      return;
    };

    match command.as_ref() {
      APUTriangleOscillatorCommand::NoOp => {}
      APUTriangleOscillatorCommand::SetEnabled(enabled) => self.enabled = *enabled,
      APUTriangleOscillatorCommand::WriteControl(value) => {
        self.linear_counter.counter = value.counter_reload_value();
        self.linear_counter.control_flag = value.control_flag();
        self.length_counter.halt = value.control_flag();
      }
      APUTriangleOscillatorCommand::WriteTimerRegister(value) => {
        self.sequencer.timer = value.timer();
      }
      APUTriangleOscillatorCommand::LoadLengthCounterByIndex(index) => {
        self.length_counter.load_length(*index);
        self.linear_counter.reload_flag = true;
      }
      APUTriangleOscillatorCommand::SetAPUSequencerMode(mode) => {
        self.timer.sequencer_mode = mode.clone();
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUTriangleChannel {
  pub control: APUTriangleControlRegister,
  pub timer: APUTimerRegister,
  pub enabled: bool,
  pub length_counter_load_index: u8,
  pub sequencer_mode: APUSequencerMode,
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
      length_counter_load_index: 0,
      sequencer_mode: APUSequencerMode::FourStep,
    }
  }

  pub fn write_control(&mut self, value: APUTriangleControlRegister) {
    self.control = value;
  }

  pub fn write_timer_byte(&mut self, value: u8, high_byte: bool) {
    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | (((value & 0b111) as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    self.timer = new_value;

    if high_byte {
      self.length_counter_load_index = value >> 3;
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUTriangleChannelState {
  enabled: bool,
  control: APUTriangleControlRegister,
  timer_register: APUTimerRegister,
  length_counter_load_index: u8,
  sequencer_mode: APUSequencerMode,
}

impl APUChannelStateTrait for APUTriangleChannelState {
  type Channel = APUTriangleChannel;
  type Command = APUTriangleOscillatorCommand;

  fn capture(channel: &Self::Channel) -> Self {
    APUTriangleChannelState {
      enabled: channel.enabled,
      control: channel.control,
      timer_register: channel.timer,
      length_counter_load_index: channel.length_counter_load_index,
      sequencer_mode: channel.sequencer_mode.clone(),
    }
  }

  fn commands(&self) -> CommandBuffer<Self> {
    array_vec!([Self::Command; COMMAND_BUFFER_SIZE] =>
      APUTriangleOscillatorCommand::SetEnabled(self.enabled),
      APUTriangleOscillatorCommand::WriteControl(self.control),
      APUTriangleOscillatorCommand::WriteTimerRegister(self.timer_register),
      APUTriangleOscillatorCommand::LoadLengthCounterByIndex(self.length_counter_load_index),
      APUTriangleOscillatorCommand::SetAPUSequencerMode(self.sequencer_mode.clone()),
    )
  }
}
