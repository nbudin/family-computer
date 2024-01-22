use std::{fmt::Debug, time::Duration};

use crate::audio::synth::SynthCommand;

use super::{
  APUNoiseChannelState, APUPulseChannelState, APUSynthChannel, APUTriangleChannelState, APU,
};

#[derive(Debug, Clone)]
pub struct APUState {
  pulse1: APUChannelState,
  pulse2: APUChannelState,
  triangle: APUChannelState,
  noise: APUChannelState,
}

impl APUState {
  pub fn capture(apu: &APU) -> APUState {
    APUState {
      pulse1: APUChannelState::capture(apu, APUSynthChannel::Pulse1),
      pulse2: APUChannelState::capture(apu, APUSynthChannel::Pulse2),
      triangle: APUChannelState::capture(apu, APUSynthChannel::Triangle),
      noise: APUChannelState::capture(apu, APUSynthChannel::Noise),
    }
  }

  pub fn commands(&self, time_since_start: Duration) -> Vec<SynthCommand<APUSynthChannel>> {
    self
      .pulse1
      .commands(time_since_start)
      .into_iter()
      .chain(self.pulse2.commands(time_since_start))
      .chain(self.triangle.commands(time_since_start))
      .chain(self.noise.commands(time_since_start))
      .collect()
  }

  pub fn diff_commands(
    &self,
    other: &APUState,
    time_since_start: Duration,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    self
      .pulse1
      .diff_commands(&other.pulse1, time_since_start)
      .into_iter()
      .chain(self.pulse2.diff_commands(&other.pulse2, time_since_start))
      .chain(
        self
          .triangle
          .diff_commands(&other.triangle, time_since_start),
      )
      .chain(self.noise.diff_commands(&other.noise, time_since_start))
      .collect()
  }
}

#[derive(Debug, Clone)]
pub enum APUChannelState {
  Pulse1(APUPulseChannelState),
  Pulse2(APUPulseChannelState),
  Triangle(APUTriangleChannelState),
  Noise(APUNoiseChannelState),
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
      APUSynthChannel::Noise => APUChannelState::Noise(APUNoiseChannelState::capture(&apu.noise)),
    }
  }

  pub fn commands(&self, time_since_start: Duration) -> Vec<SynthCommand<APUSynthChannel>> {
    match self {
      APUChannelState::Pulse1(state) => state
        .commands()
        .into_iter()
        .map(|command| {
          SynthCommand::ChannelCommand(APUSynthChannel::Pulse1, Box::new(command), time_since_start)
        })
        .collect(),
      APUChannelState::Pulse2(state) => state
        .commands()
        .into_iter()
        .map(|command| {
          SynthCommand::ChannelCommand(APUSynthChannel::Pulse2, Box::new(command), time_since_start)
        })
        .collect(),
      APUChannelState::Triangle(state) => state
        .commands()
        .into_iter()
        .map(|command| {
          SynthCommand::ChannelCommand(
            APUSynthChannel::Triangle,
            Box::new(command),
            time_since_start,
          )
        })
        .collect(),
      APUChannelState::Noise(state) => state
        .commands()
        .into_iter()
        .map(|command| {
          SynthCommand::ChannelCommand(APUSynthChannel::Noise, Box::new(command), time_since_start)
        })
        .collect(),
    }
  }

  pub fn diff_commands(
    &self,
    other: &APUChannelState,
    time_since_start: Duration,
  ) -> Vec<SynthCommand<APUSynthChannel>> {
    match self {
      APUChannelState::Pulse1(before) => {
        if let APUChannelState::Pulse1(after) = other {
          before
            .diff_commands(after)
            .into_iter()
            .map(|command| {
              SynthCommand::ChannelCommand(
                APUSynthChannel::Pulse1,
                Box::new(command),
                time_since_start,
              )
            })
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
            .map(|command| {
              SynthCommand::ChannelCommand(
                APUSynthChannel::Pulse2,
                Box::new(command),
                time_since_start,
              )
            })
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
              SynthCommand::ChannelCommand(
                APUSynthChannel::Triangle,
                Box::new(command),
                time_since_start,
              )
            })
            .collect()
        } else {
          panic!("Cannot diff Triangle channel against {:?}", other);
        }
      }
      APUChannelState::Noise(before) => {
        if let APUChannelState::Noise(after) = other {
          before
            .diff_commands(after)
            .into_iter()
            .map(|command| {
              SynthCommand::ChannelCommand(
                APUSynthChannel::Noise,
                Box::new(command),
                time_since_start,
              )
            })
            .collect()
        } else {
          panic!("Cannot diff Noise channel against {:?}", other);
        }
      }
    }
  }
}

pub trait APUChannelStateTrait: Debug {
  type Channel;
  type Command: Eq;

  fn capture(channel: &Self::Channel) -> Self
  where
    Self: Sized;
  fn commands(&self) -> Vec<Self::Command>;
  fn diff_commands(&self, after: &Self) -> Vec<Self::Command> {
    let before_commands = self.commands();
    let after_commands = after.commands();

    before_commands
      .into_iter()
      .zip(after_commands.into_iter())
      .filter_map(
        |(before, after)| {
          if before == after {
            None
          } else {
            Some(after)
          }
        },
      )
      .collect()
  }
}
