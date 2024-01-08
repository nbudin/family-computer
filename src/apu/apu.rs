use crate::{bus::Bus, nes::NES};

use super::{
  APUFrameCounterRegister, APUPulseChannel, APUSequencerMode, APUState, APUStatusRegister,
  APUTriangleChannel,
};

#[derive(Debug)]
pub struct APU {
  pub pulse1: APUPulseChannel,
  pub pulse2: APUPulseChannel,
  pub triangle: APUTriangleChannel,
  pub status: APUStatusRegister,
  pub frame_counter: APUFrameCounterRegister,
  pub cycle_count: u64,
  pub frame_cycle_count: u64,
  prev_state: Option<APUState>,
}

impl APU {
  pub fn new() -> Self {
    Self {
      pulse1: APUPulseChannel::new(),
      pulse2: APUPulseChannel::new(),
      triangle: APUTriangleChannel::new(),
      status: 0.into(),
      frame_counter: 0.into(),
      cycle_count: 0,
      frame_cycle_count: 0,
      prev_state: None,
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
        for envelope in [&mut nes.apu.pulse1.envelope, &mut nes.apu.pulse2.envelope] {
          envelope.tick();
        }
      }

      if half_frame {
        // note length and sweep adjust
        for length_counter in [
          &mut nes.apu.pulse1.length_counter,
          &mut nes.apu.pulse2.length_counter,
        ] {
          length_counter.tick();
        }
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

      let new_state = APUState::capture(&nes.apu);
      let commands = if let Some(prev_state) = &nes.apu.prev_state {
        prev_state.diff_commands(&new_state)
      } else {
        new_state.commands()
      };
      for command in commands {
        nes.apu_sender.send_blocking(command).unwrap();
      }
      nes.apu.prev_state = Some(new_state);
    }

    nes.apu.cycle_count += 1;
  }

  fn write_status_byte(&mut self, value: APUStatusRegister) {
    self.status = value;
    self.pulse1.enabled = value.pulse1_enable();
    self.pulse2.enabled = value.pulse2_enable();
    self.triangle.enabled = value.triangle_enable();
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
      0x4000 => self.pulse1.write_control(value.into()),
      0x4001 => self.pulse1.sweep = value.into(),
      0x4002 => self.pulse1.write_timer_byte(value, false),
      0x4003 => self.pulse1.write_timer_byte(value, true),
      0x4004 => self.pulse2.write_control(value.into()),
      0x4005 => self.pulse2.sweep = value.into(),
      0x4006 => self.pulse2.write_timer_byte(value, false),
      0x4007 => self.pulse2.write_timer_byte(value, true),
      0x4008 => self.triangle.write_control(value.into()),
      0x400a => self.triangle.write_timer_byte(value, false),
      0x400b => self.triangle.write_timer_byte(value, true),
      0x4015 => self.write_status_byte(value.into()),
      _ => {}
    }
  }
}
