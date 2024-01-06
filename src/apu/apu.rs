use smol::channel::Sender;

use crate::{audio::synth::SynthCommand, bus::Bus};

use super::{
  channels::{APUPulseChannel, APUTriangleChannel},
  registers::{APUFrameCounterRegister, APUStatusRegister},
  APUSynthChannel,
};

#[derive(Debug, Clone)]
pub struct APU {
  pub pulse1: APUPulseChannel,
  pub pulse2: APUPulseChannel,
  pub triangle: APUTriangleChannel,
  pub status: APUStatusRegister,
  pub frame_counter: APUFrameCounterRegister,
  pub pending_commands: Vec<SynthCommand<APUSynthChannel>>,
}

impl APU {
  pub fn new() -> Self {
    Self {
      pulse1: APUPulseChannel::new(APUSynthChannel::Pulse1),
      pulse2: APUPulseChannel::new(APUSynthChannel::Pulse2),
      triangle: APUTriangleChannel::new(),
      status: 0.into(),
      frame_counter: 0.into(),
      pending_commands: vec![],
    }
  }

  pub fn tick(&mut self, sender: &Sender<SynthCommand<APUSynthChannel>>) {
    let pending_commands = self.pending_commands.clone();
    self.pending_commands.clear();

    for command in pending_commands {
      sender.send_blocking(command).unwrap();
    }
  }
}

impl Bus<u16> for APU {
  fn try_read_readonly(&self, addr: u16) -> Option<u8> {
    match addr {
      0x4015 => Some(self.status.into()),
      _ => None,
    }
  }

  fn write(&mut self, addr: u16, value: u8) {
    match addr {
      0x4000 => self
        .pending_commands
        .extend(self.pulse1.write_control(value.into())),
      0x4001 => self.pulse1.sweep = value.into(),
      0x4002 => self
        .pending_commands
        .extend(self.pulse1.write_timer_byte(value, false)),
      0x4003 => self
        .pending_commands
        .extend(self.pulse1.write_timer_byte(value, true)),
      0x4004 => self
        .pending_commands
        .extend(self.pulse2.write_control(value.into())),
      0x4005 => self.pulse2.sweep = value.into(),
      0x4006 => self
        .pending_commands
        .extend(self.pulse2.write_timer_byte(value, false)),
      0x4007 => self
        .pending_commands
        .extend(self.pulse2.write_timer_byte(value, false)),
      0x4008 => self.triangle.control = value.into(),
      0x400a => {
        self.triangle.timer = ((u16::from(self.triangle.timer) & 0xff00) | value as u16).into()
      }
      0x400b => {
        self.triangle.timer =
          ((u16::from(self.triangle.timer) & 0x00ff) | ((value as u16) << 8)).into()
      }
      _ => {}
    }
  }
}
