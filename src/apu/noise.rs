use std::{ops::Rem, time::Duration};

use tinyvec::array_vec;

use crate::{apu::COMMAND_BUFFER_SIZE, audio::audio_channel::AudioChannel};

use super::{
  channel::APUChannel,
  envelope::APUEnvelope,
  timing::{APUOscillatorTimer, APUTimerInstant},
  APUChannelStateTrait, APUFrameCounterRegister, APULengthCounter, APUNoiseControlRegister,
  APUNoiseLengthCounterLoadRegister, APUNoiseModePeriodRegister, APUSequencer, APUSequencerMode,
  CommandBuffer,
};

#[derive(Debug, PartialEq, Eq, Hash, Default)]
pub enum APUNoiseOscillatorCommand {
  #[default]
  NoOp,
  SetShiftRegisterReload(u16),
  SetEnabled(bool),
  SetMode(bool),
  WriteControl(APUNoiseControlRegister),
  LoadLengthCounterByIndex(u8),
  SetAPUSequencerMode(APUSequencerMode),
  FrameCounterSet,
}

#[derive(Debug, Clone)]
pub struct APUNoiseOscillator {
  shift_register: APUSequencer,
  envelope: APUEnvelope,
  length_counter: APULengthCounter,
  timer: APUOscillatorTimer,
  mode: bool,
  enabled: bool,
}

impl APUNoiseOscillator {
  pub fn new() -> Self {
    let mut shift_register = APUSequencer::new();
    shift_register.sequence = 1;

    Self {
      shift_register,
      envelope: APUEnvelope::new(),
      length_counter: APULengthCounter::new(),
      timer: APUOscillatorTimer::new(),
      mode: false,
      enabled: false,
    }
  }

  fn amplitude(&self) -> f32 {
    if self.length_counter.counter > 0 && self.shift_register.timer >= 8 {
      (self.envelope.output.saturating_sub(1) as f32) / 16.0
    } else {
      0.0
    }
  }
}

impl AudioChannel for APUNoiseOscillator {
  fn mix_amplitude(&self) -> f32 {
    0.00494
  }

  fn get_next_sample(&mut self, sample_rate: f32, timestamp: Duration) -> f32 {
    self.timer.tick(timestamp);

    if !self.enabled {
      return 0.0;
    }

    let cycles = self.timer.cpu_cycle_range(sample_rate);

    for _i in cycles.cycle_count.start..cycles.cycle_count.end {
      self.shift_register.tick(self.enabled, |value| {
        let feedback_bit = if self.mode { 6 } else { 1 };
        let feedback = (value & 0b1) ^ ((value & (1 << feedback_bit)) >> feedback_bit);
        (value >> 1) | (feedback << 14)
      });
    }

    if self.timer.is_quarter_frame(sample_rate) {
      self.envelope.tick();
    }
    if self.timer.is_half_frame(sample_rate) {
      self.length_counter.tick();
    }

    self.shift_register.output as f32 * self.amplitude()
  }

  fn handle_command(&mut self, command: Box<dyn std::any::Any + Send + Sync>) {
    let Ok(command) = command.downcast::<APUNoiseOscillatorCommand>() else {
      return;
    };

    match command.as_ref() {
      APUNoiseOscillatorCommand::NoOp => {}
      APUNoiseOscillatorCommand::SetShiftRegisterReload(value) => {
        self.shift_register.reload = *value;
      }
      APUNoiseOscillatorCommand::SetEnabled(enabled) => self.enabled = *enabled,
      APUNoiseOscillatorCommand::SetMode(mode) => self.mode = *mode,
      APUNoiseOscillatorCommand::WriteControl(control) => {
        self.envelope.enabled = !control.constant_volume_envelope();
        self.envelope.volume = control.volume_envelope_divider_period() as u16;
        self.length_counter.halt = control.length_counter_halt();
      }
      APUNoiseOscillatorCommand::LoadLengthCounterByIndex(value) => {
        self.length_counter.load_length(*value);
        self.envelope.start_flag = true;
      }
      APUNoiseOscillatorCommand::SetAPUSequencerMode(mode) => {
        self.timer.sequencer_mode = mode.clone()
      }
      APUNoiseOscillatorCommand::FrameCounterSet => {
        self.envelope.tick();
        self.length_counter.tick();
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUNoiseChannel {
  pub control: APUNoiseControlRegister,
  pub mode_period: APUNoiseModePeriodRegister,
  pub enabled: bool,
  pub length_counter_load: u8,
  pub sequencer_mode: APUSequencerMode,
  envelope: APUEnvelope,
  length_counter: APULengthCounter,
  frame_counter_set: bool,
}

impl Default for APUNoiseChannel {
  fn default() -> Self {
    Self::new()
  }
}

impl APUChannel for APUNoiseChannel {
  fn playing(&self) -> bool {
    self.enabled && self.length_counter.counter > 0
  }

  fn tick<T: Clone + Rem<u64, Output = T> + PartialEq<u64>>(&mut self, now: &APUTimerInstant<T>) {
    if now.is_quarter_frame() {
      self.envelope.tick();
    }
    if now.is_half_frame() {
      self.length_counter.tick();
    }
  }
}

impl APUNoiseChannel {
  pub fn new() -> Self {
    Self {
      control: APUNoiseControlRegister::from(0),
      mode_period: APUNoiseModePeriodRegister::from(0),
      enabled: false,
      length_counter_load: 0,
      sequencer_mode: APUSequencerMode::FourStep,
      envelope: APUEnvelope::new(),
      length_counter: APULengthCounter::new(),
      frame_counter_set: false,
    }
  }

  pub fn write_control(&mut self, value: APUNoiseControlRegister) {
    self.control = value;
    self.envelope.enabled = !value.constant_volume_envelope();
    self.envelope.volume = value.volume_envelope_divider_period() as u16;
    self.length_counter.halt = value.length_counter_halt();
  }

  pub fn write_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
    if !self.enabled {
      self.length_counter.reset();
    }
  }

  pub fn write_mode_period(&mut self, value: APUNoiseModePeriodRegister) {
    self.mode_period = value;
  }

  pub fn write_length_counter_load(&mut self, value: APUNoiseLengthCounterLoadRegister) {
    if self.enabled {
      self.length_counter_load = value.length_counter_load();
      self.length_counter.load_length(self.length_counter_load);
    }
  }

  pub fn write_frame_counter(&mut self, frame_counter: APUFrameCounterRegister) {
    self.sequencer_mode = frame_counter.sequencer_mode();
    if self.sequencer_mode == APUSequencerMode::FiveStep {
      self.frame_counter_set = true;
      self.length_counter.tick();
      self.envelope.tick();
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUNoiseChannelState {
  shift_register_reload: u16,
  enabled: bool,
  mode: bool,
  control: APUNoiseControlRegister,
  sequencer_mode: APUSequencerMode,
  length_counter_load: u8,
  frame_counter_set: bool,
}

impl APUChannelStateTrait for APUNoiseChannelState {
  type Channel = APUNoiseChannel;
  type Command = APUNoiseOscillatorCommand;

  fn capture(channel: &mut Self::Channel) -> Self
  where
    Self: Sized,
  {
    let frame_counter_set = channel.frame_counter_set;
    channel.frame_counter_set = false;

    Self {
      enabled: channel.enabled,
      mode: channel.mode_period.mode(),
      shift_register_reload: match channel.mode_period.period() {
        0 => 4,
        1 => 8,
        2 => 16,
        3 => 32,
        4 => 64,
        5 => 96,
        6 => 128,
        7 => 160,
        8 => 202,
        9 => 254,
        0xa => 380,
        0xb => 508,
        0xc => 762,
        0xd => 1016,
        0xe => 2034,
        _ => 4068,
      },
      control: channel.control,
      sequencer_mode: channel.sequencer_mode.clone(),
      length_counter_load: channel.length_counter_load,
      frame_counter_set,
    }
  }

  fn commands(&self) -> CommandBuffer<Self> {
    array_vec!([Self::Command; COMMAND_BUFFER_SIZE] =>
      APUNoiseOscillatorCommand::SetShiftRegisterReload(self.shift_register_reload),
      APUNoiseOscillatorCommand::SetEnabled(self.enabled),
      APUNoiseOscillatorCommand::SetMode(self.mode),
      APUNoiseOscillatorCommand::WriteControl(self.control),
      APUNoiseOscillatorCommand::SetAPUSequencerMode(self.sequencer_mode.clone()),
      APUNoiseOscillatorCommand::LoadLengthCounterByIndex(self.length_counter_load),
      if self.frame_counter_set {
        APUNoiseOscillatorCommand::FrameCounterSet
      } else {
        APUNoiseOscillatorCommand::NoOp
      }
    )
  }
}
