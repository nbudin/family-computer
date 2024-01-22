use std::time::Duration;

use smol::channel::Sender;

use crate::{audio::synth::SynthCommand, bus::Bus};

use super::{
  APUFrameCounterRegister, APUNoiseChannel, APUPulseChannel, APUSequencerMode, APUState,
  APUStatusRegister, APUSynthChannel, APUTriangleChannel, NTSC_CPU_FREQUENCY,
};

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct APU {
  pub pulse1: APUPulseChannel,
  pub pulse2: APUPulseChannel,
  pub triangle: APUTriangleChannel,
  pub noise: APUNoiseChannel,
  pub status: APUStatusRegister,
  pub frame_counter: APUFrameCounterRegister,
  pub cycle_count: u64,
  pub frame_cycle_count: u64,
  prev_state: Option<APUState>,
}

impl Default for APU {
  fn default() -> Self {
    Self::new()
  }
}

impl APU {
  pub fn new() -> Self {
    Self {
      pulse1: APUPulseChannel::new(),
      pulse2: APUPulseChannel::new(),
      triangle: APUTriangleChannel::new(),
      noise: APUNoiseChannel::new(),
      status: 0.into(),
      frame_counter: 0.into(),
      cycle_count: 0,
      frame_cycle_count: 0,
      prev_state: None,
    }
  }

  pub fn tick(
    apu: &mut APU,
    apu_sender: &Sender<SynthCommand<APUSynthChannel>>,
    cpu_cycle_count: u64,
  ) -> bool {
    let mut quarter_frame = false;
    let mut half_frame = false;
    let mut irq_set = false;

    if apu.cycle_count % 6 == 0 {
      apu.frame_cycle_count += 1;

      match apu.frame_counter.sequencer_mode() {
        APUSequencerMode::FourStep => match apu.frame_cycle_count {
          3729 => quarter_frame = true,
          7457 => {
            quarter_frame = true;
            half_frame = true;
          }
          11186 => quarter_frame = true,
          14916 => {
            quarter_frame = true;
            half_frame = true;
            apu.frame_cycle_count = 0;
            if !apu.frame_counter.interrupt_inhibit() {
              irq_set = true;
            }
          }
          _ => {}
        },
        APUSequencerMode::FiveStep => match apu.frame_cycle_count {
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
            apu.frame_cycle_count = 0;
          }
          _ => {}
        },
      }

      let new_state = APUState::capture(apu);
      let time_since_start = Duration::from_secs_f32(cpu_cycle_count as f32 / NTSC_CPU_FREQUENCY);

      let commands = if let Some(prev_state) = &apu.prev_state {
        prev_state.diff_commands(&new_state, time_since_start)
      } else {
        new_state.commands(time_since_start)
      };
      for command in commands {
        apu_sender.send_blocking(command).unwrap();
      }
      apu.prev_state = Some(new_state);
    }

    apu.cycle_count += 1;

    irq_set
  }

  fn write_status_byte(&mut self, value: APUStatusRegister) {
    self.status = value;
    self.pulse1.enabled = value.pulse1_enable();
    self.pulse2.enabled = value.pulse2_enable();
    self.triangle.enabled = value.triangle_enable();
    self.noise.enabled = value.noise_enable();
  }

  fn write_frame_counter_byte(&mut self, value: APUFrameCounterRegister) {
    self.frame_counter = value;
    self.pulse1.sequencer_mode = value.sequencer_mode();
    self.pulse2.sequencer_mode = value.sequencer_mode();
    self.triangle.sequencer_mode = value.sequencer_mode();
    self.noise.sequencer_mode = value.sequencer_mode();
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
      0x400c => self.noise.write_control(value.into()),
      0x400e => self.noise.write_mode_period(value.into()),
      0x400f => self.noise.write_length_counter_load(value.into()),
      0x4015 => self.write_status_byte(value.into()),
      0x4017 => self.write_frame_counter_byte(value.into()),
      _ => {}
    }
  }
}
