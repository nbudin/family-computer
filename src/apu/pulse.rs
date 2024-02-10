use std::{f32::consts::PI, ops::Rem, time::Duration};

use tinyvec::array_vec;

use crate::{apu::COMMAND_BUFFER_SIZE, audio::audio_channel::AudioChannel};

use super::{
  channel::APUChannel,
  envelope::APUEnvelope,
  sweep::APUSweep,
  timing::{APUOscillatorTimer, APUTimerInstant},
  APUChannelStateTrait, APUFrameCounterRegister, APULengthCounter, APUPulseControlRegister,
  APUPulseSweepRegister, APUSequencer, APUSequencerMode, APUTimerRegister, CommandBuffer,
  MAX_PULSE_FREQUENCY,
};

const TWO_PI: f32 = PI * 2.0;

#[derive(Debug, PartialEq, Eq, Hash, Default)]
pub enum APUPulseOscillatorCommand {
  #[default]
  NoOp,
  WriteControl(APUPulseControlRegister),
  WriteSweep(APUPulseSweepRegister),
  WriteTimerRegister(APUTimerRegister),
  SetEnabled(bool),
  LoadLengthCounterByIndex(u8),
  SetAPUSequencerMode(APUSequencerMode),
  FrameCounterSet,
}

#[derive(Debug, Clone)]
pub struct APUPulseOscillator {
  harmonics: usize,
  enabled: bool,
  duty_cycle: f32,
  sequencer: APUSequencer,
  envelope: APUEnvelope,
  length_counter: APULengthCounter,
  timer: APUOscillatorTimer,
  timer_register: APUTimerRegister,
  sweep: APUSweep,
}

impl APUPulseOscillator {
  pub fn new(ones_complement_negate: bool) -> Self {
    Self {
      harmonics: 20,
      enabled: false,
      duty_cycle: 0.5,
      sequencer: APUSequencer::new(),
      envelope: APUEnvelope::new(),
      length_counter: APULengthCounter::new(),
      timer: APUOscillatorTimer::new(),
      timer_register: APUTimerRegister::from(0),
      sweep: APUSweep::new(ones_complement_negate),
    }
  }

  fn amplitude(&self) -> f32 {
    if self.length_counter.counter > 0
      // && self.sequencer.timer >= 8
      // && self.sweep.enabled
      && self.envelope.output > 2
      && self.timer_register.pulse_frequency() < MAX_PULSE_FREQUENCY
    {
      f32::from(self.envelope.output - 1) / 16.0
    } else {
      0.0
    }
  }
}

impl AudioChannel for APUPulseOscillator {
  fn mix_amplitude(&self) -> f32 {
    0.00752
  }

  fn get_next_sample(&mut self, sample_rate: f32, timestamp: Duration) -> f32 {
    // let prev_cycles = self.timer.cpu_cycle_count(sample_rate);
    self.timer.tick(timestamp);

    if !self.enabled {
      return 0.0;
    }

    // let cycles = self.timer.cpu_cycle_count(sample_rate);
    // let elapsed_cycles = cycles - prev_cycles;

    // for _i in 0..elapsed_cycles {
    //   self.sequencer.tick(self.enabled, |sequence| {
    //     ((sequence & 0x0001) << 7) | ((sequence & 0x00fe) >> 1)
    //   });
    // }

    if self.timer.is_quarter_frame(sample_rate) {
      self.envelope.tick();
    }
    if self.timer.is_half_frame(sample_rate) {
      self.length_counter.tick();
      self.sweep.tick(&self.timer_register);
    }

    let mut wave1: f32 = 0.0;
    let mut wave2: f32 = 0.0;
    let p = self.duty_cycle * TWO_PI;
    let current_sample_index = self.timer.current_sample_index(sample_rate);
    let frequency = self.sweep.output.pulse_frequency();

    for n in 1..(self.harmonics + 1) {
      let n = n as f32;
      let sample_index_radians = n * frequency * TWO_PI * (current_sample_index / sample_rate);
      wave1 += -(sample_index_radians).sin() / n;
      wave2 += -(sample_index_radians - (p * n)).sin() / n;
    }

    (2.0 * self.amplitude() / PI) * (wave1 - wave2)
  }

  fn handle_command(&mut self, command: Box<dyn std::any::Any + Send + Sync>) {
    let Ok(command) = command.downcast::<APUPulseOscillatorCommand>() else {
      return;
    };

    match command.as_ref() {
      APUPulseOscillatorCommand::NoOp => {}
      APUPulseOscillatorCommand::SetEnabled(enabled) => {
        self.enabled = *enabled;
        if !self.enabled {
          self.length_counter.reset();
        }
      }
      APUPulseOscillatorCommand::WriteControl(value) => {
        self.duty_cycle = value.duty_cycle_float();
        self.sequencer.sequence = value.duty_cycle_sequence() as u32;
        self.length_counter.halt = value.length_counter_halt();
        self.envelope.loop_flag = value.length_counter_halt();
        self.envelope.enabled = !value.constant_volume_envelope();
        self.envelope.volume = value.volume_envelope_divider_period() as u16;
      }
      APUPulseOscillatorCommand::WriteSweep(value) => {
        self.sweep.enabled = value.enabled();
        self.sweep.divider_period = value.divider_period();
        self.sweep.negate = value.negate();
        self.sweep.shift_count = value.shift_count();
        self.sweep.reload_flag = true;
      }
      APUPulseOscillatorCommand::WriteTimerRegister(value) => {
        self.timer_register = value.clone();
      }
      APUPulseOscillatorCommand::LoadLengthCounterByIndex(index) => {
        self.length_counter.load_length(*index);
        self.envelope.start_flag = true;
      }
      APUPulseOscillatorCommand::SetAPUSequencerMode(mode) => {
        self.timer.sequencer_mode = mode.clone();
      }
      APUPulseOscillatorCommand::FrameCounterSet => {
        self.envelope.tick();
        self.length_counter.tick();
        self.sweep.tick(&self.timer_register);
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUPulseChannel {
  pub control: APUPulseControlRegister,
  pub sweep: APUPulseSweepRegister,
  pub timer: APUTimerRegister,
  pub enabled: bool,
  pub length_counter_load_index: u8,
  pub sequencer_mode: APUSequencerMode,
  pub frame_counter_set: bool,
  envelope: APUEnvelope,
  length_counter: APULengthCounter,
}

impl Default for APUPulseChannel {
  fn default() -> Self {
    Self::new()
  }
}

impl APUChannel for APUPulseChannel {
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

impl APUPulseChannel {
  pub fn new() -> Self {
    Self {
      control: 0.into(),
      sweep: 0.into(),
      timer: 0.into(),
      enabled: false,
      length_counter_load_index: 0,
      sequencer_mode: APUSequencerMode::default(),
      envelope: APUEnvelope::new(),
      length_counter: APULengthCounter::new(),
      frame_counter_set: false,
    }
  }

  pub fn write_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
    if !self.enabled {
      self.length_counter.reset();
    }
  }

  pub fn write_control(&mut self, value: APUPulseControlRegister) {
    self.control = value;
    self.length_counter.halt = value.length_counter_halt();
    self.envelope.loop_flag = value.length_counter_halt();
    self.envelope.enabled = !value.constant_volume_envelope();
    self.envelope.volume = value.volume_envelope_divider_period() as u16;
    self.envelope.tick();
  }

  pub fn write_sweep(&mut self, value: APUPulseSweepRegister) {
    self.sweep = value;
  }

  pub fn write_timer_byte(&mut self, value: u8, high_byte: bool) {
    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | (((value & 0b111) as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    self.timer = new_value;

    if high_byte && self.enabled {
      self.length_counter_load_index = value >> 3;
      self
        .length_counter
        .load_length(self.length_counter_load_index);
      self.envelope.start_flag = true;
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
pub struct APUPulseChannelState {
  control: APUPulseControlRegister,
  enabled: bool,
  length_counter_load_index: u8,
  sequencer_mode: APUSequencerMode,
  timer_register: APUTimerRegister,
  sweep: APUPulseSweepRegister,
  frame_counter_set: bool,
}

impl APUChannelStateTrait for APUPulseChannelState {
  type Channel = APUPulseChannel;
  type Command = APUPulseOscillatorCommand;

  fn capture(channel: &mut Self::Channel) -> Self {
    let frame_counter_set = channel.frame_counter_set;
    channel.frame_counter_set = false;

    APUPulseChannelState {
      enabled: channel.enabled,
      control: channel.control,
      length_counter_load_index: channel.length_counter_load_index,
      sequencer_mode: channel.sequencer_mode.clone(),
      timer_register: channel.timer.clone(),
      sweep: channel.sweep.clone(),
      frame_counter_set,
    }
  }

  fn commands(&self) -> CommandBuffer<Self> {
    array_vec!([Self::Command; COMMAND_BUFFER_SIZE] =>
      APUPulseOscillatorCommand::SetEnabled(self.enabled),
      APUPulseOscillatorCommand::WriteControl(self.control),
      APUPulseOscillatorCommand::WriteSweep(self.sweep),
      APUPulseOscillatorCommand::WriteTimerRegister(self.timer_register),
      APUPulseOscillatorCommand::LoadLengthCounterByIndex(self.length_counter_load_index),
      APUPulseOscillatorCommand::SetAPUSequencerMode(self.sequencer_mode.clone()),
      if self.frame_counter_set {
        APUPulseOscillatorCommand::FrameCounterSet
      } else {
        APUPulseOscillatorCommand::NoOp
      }
    )
  }
}
