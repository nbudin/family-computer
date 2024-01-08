use std::fmt::Debug;

use crate::audio::{oscillator::OscillatorCommand, synth::SynthCommand};

use super::{APUPulseChannel, APUPulseOscillatorCommand, APUSynthChannel, APUTriangleChannel, APU};

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
  amplitude: f32,
  frequency: f32,
  enabled: bool,
}

impl APUChannelStateTrait for APUTriangleChannelState {
  type Channel = APUTriangleChannel;
  type Command = OscillatorCommand;

  fn capture(channel: &Self::Channel) -> Self {
    APUTriangleChannelState {
      amplitude: 1.0,
      frequency: channel.timer.triangle_frequency(),
      enabled: true,
    }
  }

  fn diff_commands(&self, after: &APUTriangleChannelState) -> Vec<OscillatorCommand> {
    let mut commands: Vec<OscillatorCommand> = vec![];

    if self.amplitude != after.amplitude {
      commands.push(OscillatorCommand::SetAmplitude(after.amplitude))
    }

    if self.frequency != after.frequency {
      commands.push(OscillatorCommand::SetFrequency(after.frequency))
    }

    // if self.enabled != after.enabled {
    //   commands.push(OscillatorCommand::SetEnabled(after.enabled))
    // }

    commands
  }
}
