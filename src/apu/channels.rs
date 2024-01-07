use crate::audio::{oscillator::OscillatorCommand, synth::SynthCommand};

use super::{
  registers::{
    APUPulseControlRegister, APUPulseSweepRegister, APUTimerRegister, APUTriangleControlRegister,
  },
  APUSequencer, APUSynthChannel,
};

#[derive(Debug, Clone)]
pub struct APUPulseChannel {
  pub control: APUPulseControlRegister,
  pub sweep: APUPulseSweepRegister,
  pub timer: APUTimerRegister,
  pub sequencer: APUSequencer,
  synth_channel: APUSynthChannel,
}

impl APUPulseChannel {
  pub fn new(synth_channel: APUSynthChannel) -> Self {
    Self {
      control: 0.into(),
      sweep: 0.into(),
      timer: 0.into(),
      sequencer: APUSequencer {
        output: 0,
        reload: 0,
        sequence: 0,
        timer: 0,
      },
      synth_channel,
    }
  }

  pub fn write_control(
    &mut self,
    value: APUPulseControlRegister,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    let mut commands: Vec<SynthCommand<APUSynthChannel>> = vec![];

    if self.control.duty_cycle() != value.duty_cycle() {
      commands.push(SynthCommand::ChannelCommand(
        self.synth_channel,
        Box::new(OscillatorCommand::SetDutyCycle(value.duty_cycle_float())),
      ))
    }

    if self.control.volume_envelope_divider_period() != value.volume_envelope_divider_period() {
      commands.push(SynthCommand::ChannelCommand(
        self.synth_channel,
        Box::new(OscillatorCommand::SetAmplitude(value.amplitude())),
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
      commands.push(SynthCommand::ChannelCommand(
        self.synth_channel,
        Box::new(OscillatorCommand::SetFrequency(new_value.pulse_frequency())),
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
      commands.push(SynthCommand::ChannelCommand(
        APUSynthChannel::Triangle,
        Box::new(OscillatorCommand::SetFrequency(new_value.pulse_frequency())),
      ))
    }

    self.timer = new_value;
    commands
  }
}
