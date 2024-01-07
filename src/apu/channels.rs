use crate::audio::{oscillator::OscillatorCommand, synth::SynthCommand};

use super::{
  registers::{
    APUPulseControlRegister, APUPulseSweepRegister, APUTimerRegister, APUTriangleControlRegister,
  },
  APUSynthChannel,
};

#[derive(Debug, Clone)]
pub struct APUPulseChannel {
  pub control: APUPulseControlRegister,
  pub sweep: APUPulseSweepRegister,
  pub timer: APUTimerRegister,
  oscillator_index: APUSynthChannel,
}

impl APUPulseChannel {
  pub fn new(oscillator_index: APUSynthChannel) -> Self {
    Self {
      control: 0.into(),
      sweep: 0.into(),
      timer: 0.into(),
      oscillator_index,
    }
  }

  pub fn write_control(
    &mut self,
    value: APUPulseControlRegister,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    let mut commands: Vec<SynthCommand<APUSynthChannel>> = vec![];

    if self.control.duty_cycle() != value.duty_cycle() {
      commands.push(SynthCommand::OscillatorCommand(
        self.oscillator_index,
        OscillatorCommand::SetDutyCycle(value.duty_cycle_float()),
      ))
    }

    if self.control.volume_envelope_divider_period() != value.volume_envelope_divider_period() {
      commands.push(SynthCommand::OscillatorCommand(
        self.oscillator_index,
        OscillatorCommand::SetAmplitude(value.amplitude()),
      ))
    }

    self.control = value;
    commands
  }

  pub fn write_timer_byte(
    &mut self,
    value: u8,
    high_byte: bool,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    let mut commands: Vec<SynthCommand<APUSynthChannel>> = vec![];

    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | ((value as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    if self.timer.timer() != new_value.timer() {
      commands.push(SynthCommand::OscillatorCommand(
        self.oscillator_index,
        OscillatorCommand::SetFrequency(new_value.pulse_frequency()),
      ))
    }

    self.timer = new_value;
    commands
  }
}

#[derive(Debug, Clone)]
pub struct APUTriangleChannel {
  pub control: APUTriangleControlRegister,
  pub timer: APUTimerRegister,
}

impl APUTriangleChannel {
  pub fn new() -> Self {
    Self {
      control: 0.into(),
      timer: 0.into(),
    }
  }

  pub fn write_control(
    &mut self,
    value: APUTriangleControlRegister,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    let commands: Vec<SynthCommand<APUSynthChannel>> = vec![];

    self.control = value;
    commands
  }

  pub fn write_timer_byte(
    &mut self,
    value: u8,
    high_byte: bool,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    let mut commands: Vec<SynthCommand<APUSynthChannel>> = vec![];

    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | ((value as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    if self.timer.timer() != new_value.timer() {
      commands.push(SynthCommand::OscillatorCommand(
        APUSynthChannel::Triangle,
        OscillatorCommand::SetFrequency(new_value.pulse_frequency()),
      ))
    }

    self.timer = new_value;
    commands
  }
}
