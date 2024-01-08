use std::f32::consts::PI;

use fastapprox::fast::sinfull;

use crate::audio::audio_channel::AudioChannel;

use super::{
  envelope::APUEnvelope, APULengthCounter, APUPulseControlRegister, APUPulseSweepRegister,
  APUSequencer, APUTimerRegister,
};

const TWO_PI: f32 = PI * 2.0;

#[derive(Debug)]
pub enum APUPulseOscillatorCommand {
  SetDutyCycle(f32),
  SetFrequency(f32),
  SetAmplitude(f32),
  SetEnabled(bool),
}

#[derive(Clone)]
pub struct APUPulseOscillator {
  current_sample_index: f32,
  duty_cycle: f32,
  frequency: f32,
  amplitude: f32,
  harmonics: usize,
  enabled: bool,
}

impl APUPulseOscillator {
  pub fn new() -> Self {
    Self {
      current_sample_index: 0.0,
      duty_cycle: 0.5,
      frequency: 440.0,
      amplitude: 0.0,
      harmonics: 20,
      enabled: false,
    }
  }
}

impl AudioChannel for APUPulseOscillator {
  fn get_next_sample(&mut self, sample_rate: f32) -> f32 {
    self.current_sample_index = (self.current_sample_index + 1.0) % sample_rate;

    if !self.enabled {
      return 0.0;
    }

    let mut wave1: f32 = 0.0;
    let mut wave2: f32 = 0.0;
    let p = self.duty_cycle * TWO_PI;

    for n in 1..(self.harmonics + 1) {
      let n = n as f32;
      let sample_index_radians =
        (n * self.frequency * TWO_PI * self.current_sample_index) / sample_rate;
      wave1 += -sinfull(sample_index_radians) / n;
      wave2 += -sinfull(sample_index_radians - (p * n)) / n;
    }

    (2.0 * self.amplitude / PI) * (wave1 - wave2)
  }

  fn handle_command(&mut self, command: Box<dyn std::any::Any + Send + Sync>) {
    let Ok(command) = command.downcast::<APUPulseOscillatorCommand>() else {
      return;
    };

    match command.as_ref() {
      APUPulseOscillatorCommand::SetDutyCycle(duty_cycle) => self.duty_cycle = *duty_cycle,
      APUPulseOscillatorCommand::SetAmplitude(amplitude) => self.amplitude = *amplitude,
      APUPulseOscillatorCommand::SetFrequency(frequency) => self.frequency = *frequency,
      APUPulseOscillatorCommand::SetEnabled(enabled) => self.enabled = *enabled,
    }
  }
}

#[derive(Debug, Clone)]
pub struct APUPulseChannel {
  pub control: APUPulseControlRegister,
  pub sweep: APUPulseSweepRegister,
  pub timer: APUTimerRegister,
  pub sequencer: APUSequencer,
  pub enabled: bool,
  pub envelope: APUEnvelope,
  pub length_counter: APULengthCounter,
}

impl APUPulseChannel {
  pub fn new() -> Self {
    Self {
      control: 0.into(),
      sweep: 0.into(),
      timer: 0.into(),
      enabled: true,
      sequencer: APUSequencer {
        output: 0,
        reload: 0,
        sequence: 0,
        timer: 0,
      },
      envelope: APUEnvelope::new(),
      length_counter: APULengthCounter::new(),
    }
  }

  pub fn amplitude(&self) -> f32 {
    if self.length_counter.counter > 0
      && self.sequencer.timer >= 8
      // && self.sweep.enabled()
      && self.envelope.output > 2
    {
      f32::from(self.envelope.output - 1) / 16.0
    } else {
      0.0
    }
  }

  pub fn duty_cycle_float(&self) -> f32 {
    self.control.duty_cycle_float()
  }

  pub fn frequency(&self) -> f32 {
    self.timer.pulse_frequency()
  }

  pub fn write_control(&mut self, value: APUPulseControlRegister) {
    self.sequencer.sequence = value.duty_cycle_sequence() as u32;
    self.envelope.loop_flag = value.length_counter_halt();
    self.control = value;
  }

  pub fn write_timer_byte(&mut self, value: u8, high_byte: bool) {
    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | ((value as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    self.timer = new_value;
  }
}
