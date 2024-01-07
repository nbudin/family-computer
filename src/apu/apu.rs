use crate::{audio::synth::SynthCommand, bus::Bus, nes::NES};

use super::{
  APUFrameCounterRegister, APUPulseChannel, APUSequencerMode, APUStatusRegister, APUSynthChannel,
  APUTriangleChannel,
};

#[derive(Debug)]
pub struct APU {
  pub pulse1: APUPulseChannel,
  pub pulse2: APUPulseChannel,
  pub triangle: APUTriangleChannel,
  pub status: APUStatusRegister,
  pub frame_counter: APUFrameCounterRegister,
  pub pending_commands: Vec<SynthCommand<APUSynthChannel>>,
  pub cycle_count: u64,
  pub frame_cycle_count: u64,
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
      cycle_count: 0,
      frame_cycle_count: 0,
    }
  }

  pub fn tick(nes: &mut NES) {
    let mut quarter_frame = false;
    let mut half_frame = false;

    if nes.apu.cycle_count % 6 == 0 {
      nes.apu.frame_cycle_count += 1;

      match nes.apu.frame_counter.sequencer_mode() {
        APUSequencerMode::FourStep => match nes.apu.frame_cycle_count {
          3729 => quarter_frame = true,
          7457 => {
            quarter_frame = true;
            half_frame = true;
          }
          11186 => quarter_frame = true,
          14916 => {
            quarter_frame = true;
            half_frame = true;
            nes.apu.frame_cycle_count = 0;
          }
          _ => {}
        },
        APUSequencerMode::FiveStep => match nes.apu.frame_cycle_count {
          3729 => quarter_frame = true,
          7457 => {
            quarter_frame = true;
            half_frame = true;
          }
          11186 => quarter_frame = true,
          14916 => {}
          18641 => {
            quarter_frame = true;
            half_frame = true;
            nes.apu.frame_cycle_count = 0;
          }
          _ => {}
        },
      }

      if quarter_frame {
        // volume envelope adjust
      }

      if half_frame {
        // note length and sweep adjust
      }

      nes
        .apu
        .pulse1
        .sequencer
        .tick(nes.apu.status.pulse1_enable(), |sequence| {
          ((sequence & 0x0001) << 7) | ((sequence & 0x00fe) >> 1)
        });

      nes
        .apu
        .pulse2
        .sequencer
        .tick(nes.apu.status.pulse1_enable(), |sequence| {
          ((sequence & 0x0001) << 7) | ((sequence & 0x00fe) >> 1)
        });
    }

    let pending_commands = std::mem::take(&mut nes.apu.pending_commands);

    for command in pending_commands {
      nes.apu_sender.send_blocking(command).unwrap();
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
        .extend(self.pulse2.write_timer_byte(value, true)),
      0x4008 => self
        .pending_commands
        .extend(self.triangle.write_control(value.into())),
      0x400a => self
        .pending_commands
        .extend(self.triangle.write_timer_byte(value, false)),
      0x400b => self
        .pending_commands
        .extend(self.triangle.write_timer_byte(value, true)),
      _ => {}
    }
  }
}
