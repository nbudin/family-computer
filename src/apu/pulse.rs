use std::{f32::consts::PI, time::Duration};

use fastapprox::fast::sinfull;

use crate::audio::audio_channel::AudioChannel;

use super::{
  envelope::APUEnvelope, timing::APUOscillatorTimer, APUChannelStateTrait, APULengthCounter,
  APUPulseControlRegister, APUPulseSweepRegister, APUSequencer, APUSequencerMode, APUTimerRegister,
  MAX_PULSE_FREQUENCY,
};

const TWO_PI: f32 = PI * 2.0;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum APUPulseOscillatorCommand {
  WriteControl(APUPulseControlRegister),
  SetEnabled(bool),
  LoadLengthCounterByIndex(u8),
  SetAPUSequencerMode(APUSequencerMode),
}

#[derive(Clone)]
pub struct APUPulseOscillator {
  harmonics: usize,
  enabled: bool,
  duty_cycle: f32,
  sequencer: APUSequencer,
  envelope: APUEnvelope,
  length_counter: APULengthCounter,
  timer: APUOscillatorTimer,
  timer_register: APUTimerRegister,
}

impl APUPulseOscillator {
  pub fn new() -> Self {
    Self {
      harmonics: 20,
      enabled: false,
      duty_cycle: 0.5,
      sequencer: APUSequencer::new(),
      envelope: APUEnvelope::new(),
      length_counter: APULengthCounter::new(),
      timer: APUOscillatorTimer::new(),
      timer_register: APUTimerRegister::from(0),
    }
  }

  fn amplitude(&self) -> f32 {
    if self.length_counter.counter > 0
      // && self.sequencer.timer >= 8
      // && self.sweep.enabled()
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
    }

    let mut wave1: f32 = 0.0;
    let mut wave2: f32 = 0.0;
    let p = self.duty_cycle * TWO_PI;
    let current_sample_index = self.timer.current_sample_index(sample_rate);

    for n in 1..(self.harmonics + 1) {
      let n = n as f32;
      let sample_index_radians =
        (n * self.timer_register.pulse_frequency() * TWO_PI * current_sample_index) / sample_rate;
      wave1 += -sinfull(sample_index_radians) / n;
      wave2 += -sinfull(sample_index_radians - (p * n)) / n;
    }

    (2.0 * self.amplitude() / PI) * (wave1 - wave2)
  }

  fn handle_command(&mut self, command: Box<dyn std::any::Any + Send + Sync>) {
    let Ok(command) = command.downcast::<APUPulseOscillatorCommand>() else {
      return;
    };

    println!("{:?}", command);

    match command.as_ref() {
      APUPulseOscillatorCommand::SetEnabled(enabled) => self.enabled = *enabled,
      APUPulseOscillatorCommand::WriteControl(value) => {
        self.duty_cycle = value.duty_cycle_float();
        self.sequencer.sequence = value.duty_cycle_sequence() as u32;
        self.length_counter.halt = value.length_counter_halt();
        self.envelope.loop_flag = value.length_counter_halt();
        self.envelope.enabled = !value.constant_volume_envelope();
        self.envelope.volume = value.volume_envelope_divider_period() as u16;
      }
      APUPulseOscillatorCommand::LoadLengthCounterByIndex(index) => {
        self.length_counter.load_length(*index);
        self.envelope.start_flag = true;
      }
      APUPulseOscillatorCommand::SetAPUSequencerMode(mode) => {
        self.timer.sequencer_mode = mode.clone();
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
}

impl Default for APUPulseChannel {
  fn default() -> Self {
    Self::new()
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
    }
  }

  pub fn write_control(&mut self, value: APUPulseControlRegister) {
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
pub struct APUPulseChannelState {
  control: APUPulseControlRegister,
  enabled: bool,
  length_counter_load_index: u8,
  sequencer_mode: APUSequencerMode,
}

impl APUChannelStateTrait for APUPulseChannelState {
  type Channel = APUPulseChannel;
  type Command = APUPulseOscillatorCommand;

  fn capture(channel: &Self::Channel) -> Self {
    APUPulseChannelState {
      enabled: channel.enabled,
      control: channel.control,
      length_counter_load_index: channel.length_counter_load_index,
      sequencer_mode: channel.sequencer_mode.clone(),
    }
  }

  fn commands(&self) -> Vec<Self::Command> {
    vec![
      APUPulseOscillatorCommand::SetEnabled(self.enabled),
      APUPulseOscillatorCommand::WriteControl(self.control),
      APUPulseOscillatorCommand::LoadLengthCounterByIndex(self.length_counter_load_index),
      APUPulseOscillatorCommand::SetAPUSequencerMode(self.sequencer_mode.clone()),
    ]
  }
}
