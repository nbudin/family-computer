use std::fmt::Debug;

use crate::{apu::APUTriangleOscillatorCommand, audio::synth::SynthCommand};

use super::{APUPulseChannel, APUPulseOscillatorCommand, APUSynthChannel, APUTriangleChannel, APU};

#[derive(Debug)]
pub struct APUState {
  pulse1: APUChannelState,
  pulse2: APUChannelState,
  triangle: APUChannelState,
}

impl APUState {
  pub fn capture(apu: &APU) -> APUState {
    APUState {
      pulse1: APUChannelState::capture(apu, APUSynthChannel::Pulse1),
      pulse2: APUChannelState::capture(apu, APUSynthChannel::Pulse2),
      triangle: APUChannelState::capture(apu, APUSynthChannel::Triangle),
    }
  }

  pub fn commands(&self) -> Vec<SynthCommand<APUSynthChannel>> {
    self
      .pulse1
      .commands()
      .into_iter()
      .chain(self.pulse2.commands().into_iter())
      .chain(self.triangle.commands().into_iter())
      .collect()
  }

  pub fn diff_commands(&self, other: &APUState) -> Vec<SynthCommand<APUSynthChannel>> {
    self
      .pulse1
      .diff_commands(&other.pulse1)
      .into_iter()
      .chain(self.pulse2.diff_commands(&other.pulse2).into_iter())
      .chain(self.triangle.diff_commands(&other.triangle).into_iter())
      .collect()
  }
}

#[derive(Debug)]
pub enum APUChannelState {
  Pulse1(APUPulseChannelState),
  Pulse2(APUPulseChannelState),
  Triangle(APUTriangleChannelState),
}

impl APUChannelState {
  pub fn capture(apu: &APU, channel: APUSynthChannel) -> APUChannelState {
    match channel {
      APUSynthChannel::Pulse1 => {
        APUChannelState::Pulse1(APUPulseChannelState::capture(&apu.pulse1))
      }
      APUSynthChannel::Pulse2 => {
        APUChannelState::Pulse2(APUPulseChannelState::capture(&apu.pulse2))
      }
      APUSynthChannel::Triangle => {
        APUChannelState::Triangle(APUTriangleChannelState::capture(&apu.triangle))
      }
    }
  }

  pub fn commands(&self) -> Vec<SynthCommand<APUSynthChannel>> {
    match self {
      APUChannelState::Pulse1(state) => state
        .commands()
        .into_iter()
        .map(|command| SynthCommand::ChannelCommand(APUSynthChannel::Pulse1, Box::new(command)))
        .collect(),
      APUChannelState::Pulse2(state) => state
        .commands()
        .into_iter()
        .map(|command| SynthCommand::ChannelCommand(APUSynthChannel::Pulse2, Box::new(command)))
        .collect(),
      APUChannelState::Triangle(state) => state
        .commands()
        .into_iter()
        .map(|command| SynthCommand::ChannelCommand(APUSynthChannel::Triangle, Box::new(command)))
        .collect(),
    }
  }

  pub fn diff_commands(&self, other: &APUChannelState) -> Vec<SynthCommand<APUSynthChannel>> {
    match self {
      APUChannelState::Pulse1(before) => {
        if let APUChannelState::Pulse1(after) = other {
          before
            .diff_commands(after)
            .into_iter()
            .map(|command| SynthCommand::ChannelCommand(APUSynthChannel::Pulse1, Box::new(command)))
            .collect()
        } else {
          panic!("Cannot diff Pulse1 channel against {:?}", other);
        }
      }
      APUChannelState::Pulse2(before) => {
        if let APUChannelState::Pulse2(after) = other {
          before
            .diff_commands(after)
            .into_iter()
            .map(|command| SynthCommand::ChannelCommand(APUSynthChannel::Pulse2, Box::new(command)))
            .collect()
        } else {
          panic!("Cannot diff Pulse2 channel against {:?}", other);
        }
      }
      APUChannelState::Triangle(before) => {
        if let APUChannelState::Triangle(after) = other {
          before
            .diff_commands(after)
            .into_iter()
            .map(|command| {
              SynthCommand::ChannelCommand(APUSynthChannel::Triangle, Box::new(command))
            })
            .collect()
        } else {
          panic!("Cannot diff Triangle channel against {:?}", other);
        }
      }
    }
  }
}

pub trait APUChannelStateTrait: Debug {
  type Channel;
  type Command;

  fn capture(channel: &Self::Channel) -> Self
  where
    Self: Sized;
  fn commands(&self) -> Vec<Self::Command>;
  fn diff_commands(&self, after: &Self) -> Vec<Self::Command>;
}

#[derive(Debug, Clone)]
pub struct APUPulseChannelState {
  duty_cycle: f32,
  amplitude: f32,
  frequency: f32,
  enabled: bool,
}

impl APUChannelStateTrait for APUPulseChannelState {
  type Channel = APUPulseChannel;
  type Command = APUPulseOscillatorCommand;

  fn capture(channel: &Self::Channel) -> Self {
    APUPulseChannelState {
      duty_cycle: channel.duty_cycle_float(),
      amplitude: channel.amplitude(),
      frequency: channel.frequency(),
      enabled: channel.enabled,
    }
  }

  fn commands(&self) -> Vec<Self::Command> {
    vec![
      APUPulseOscillatorCommand::SetDutyCycle(self.duty_cycle),
      APUPulseOscillatorCommand::SetAmplitude(self.amplitude),
      APUPulseOscillatorCommand::SetFrequency(self.frequency),
      APUPulseOscillatorCommand::SetEnabled(self.enabled),
    ]
  }

  fn diff_commands(&self, after: &APUPulseChannelState) -> Vec<APUPulseOscillatorCommand> {
    let mut commands: Vec<APUPulseOscillatorCommand> = vec![];

    if self.duty_cycle != after.duty_cycle {
      commands.push(APUPulseOscillatorCommand::SetDutyCycle(after.duty_cycle))
    }

    if self.amplitude != after.amplitude {
      commands.push(APUPulseOscillatorCommand::SetAmplitude(after.amplitude))
    }

    if self.frequency != after.frequency {
      commands.push(APUPulseOscillatorCommand::SetFrequency(after.frequency))
    }

    if self.enabled != after.enabled {
      commands.push(APUPulseOscillatorCommand::SetEnabled(after.enabled))
    }

    commands
  }
}

#[derive(Debug, Clone)]
pub struct APUTriangleChannelState {
  frequency: f32,
  enabled: bool,
}

impl APUChannelStateTrait for APUTriangleChannelState {
  type Channel = APUTriangleChannel;
  type Command = APUTriangleOscillatorCommand;

  fn capture(channel: &Self::Channel) -> Self {
    APUTriangleChannelState {
      frequency: channel.timer.triangle_frequency(),
      enabled: channel.enabled,
    }
  }

  fn commands(&self) -> Vec<Self::Command> {
    vec![
      APUTriangleOscillatorCommand::SetEnabled(self.enabled),
      APUTriangleOscillatorCommand::SetFrequency(self.frequency),
    ]
  }

  fn diff_commands(&self, after: &APUTriangleChannelState) -> Vec<APUTriangleOscillatorCommand> {
    let mut commands: Vec<APUTriangleOscillatorCommand> = vec![];

    if self.frequency != after.frequency {
      commands.push(APUTriangleOscillatorCommand::SetFrequency(after.frequency))
    }

    if self.enabled != after.enabled {
      commands.push(APUTriangleOscillatorCommand::SetEnabled(after.enabled))
    }

    commands
  }
}
