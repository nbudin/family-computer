use std::time::Duration;

use smol::channel::Sender;

use crate::{audio::synth::SynthCommand, bus::Bus};

use super::{
  channel::APUChannel,
  timing::{APUTimerInstant, CycleCountRange},
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
  prev_state: Option<APUState>,
  prev_cpu_cycle_count: u64,
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
      prev_state: None,
      prev_cpu_cycle_count: 0,
    }
  }

  pub fn tick(
    apu: &mut APU,
    apu_sender: &Sender<SynthCommand<APUSynthChannel>>,
    cpu_cycle_count: u64,
  ) -> bool {
    let mut irq_set = false;

    if cpu_cycle_count % 6 == 0 {
      let instant = APUTimerInstant {
        cycle_count: CycleCountRange {
          start: apu.prev_cpu_cycle_count,
          end: cpu_cycle_count,
        },
        sequencer_mode: apu.frame_counter.sequencer_mode(),
      }
      .frame_normalized();
      apu.prev_cpu_cycle_count = cpu_cycle_count;

      if instant.cycle_count == instant.cycles_per_frame() - 1
        && instant.sequencer_mode == APUSequencerMode::FourStep
        && !apu.frame_counter.interrupt_inhibit()
      {
        irq_set = true;
      }

      apu.pulse1.tick(&instant);
      apu.pulse2.tick(&instant);
      apu.triangle.tick(&instant);
      apu.noise.tick(&instant);

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

    irq_set
  }

  fn write_status_byte(&mut self, value: APUStatusRegister) {
    self.status = value;
    self.pulse1.write_enabled(value.pulse1_enable());
    self.pulse2.write_enabled(value.pulse2_enable());
    self.triangle.write_enabled(value.triangle_enable());
    self.noise.write_enabled(value.noise_enable());
  }

  fn write_frame_counter_byte(&mut self, value: APUFrameCounterRegister) {
    self.frame_counter = value;
    self.pulse1.write_frame_counter(value);
    self.pulse2.write_frame_counter(value);
    self.triangle.write_frame_counter(value);
    self.noise.write_frame_counter(value);
  }
}

impl Bus<u16> for APU {
  fn try_read_readonly(&self, addr: u16) -> Option<u8> {
    let result = match addr {
      0x4015 => Some(
        APUStatusRegister::new()
          .with_pulse1_enable(self.pulse1.playing())
          .with_pulse2_enable(self.pulse2.playing())
          .with_triangle_enable(self.triangle.playing())
          .with_noise_enable(self.noise.playing())
          .with_frame_interrupt(self.status.frame_interrupt())
          .into(),
      ),
      _ => None,
    };

    result
  }

  fn write(&mut self, addr: u16, value: u8) {
    match addr {
      0x4000 => self.pulse1.write_control(value.into()),
      0x4001 => self.pulse1.write_sweep(value.into()),
      0x4002 => self.pulse1.write_timer_byte(value, false),
      0x4003 => self.pulse1.write_timer_byte(value, true),
      0x4004 => self.pulse2.write_control(value.into()),
      0x4005 => self.pulse2.write_sweep(value.into()),
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
