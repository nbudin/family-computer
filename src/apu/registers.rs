use bitfield_struct::bitfield;

pub const NTSC_CPU_FREQUENCY: f32 = 1.789773 * 1_000_000.0;
pub const MAX_PULSE_FREQUENCY: f32 = 13_000.0;

#[bitfield(u8)]
pub struct APUPulseControlRegister {
  #[bits(4)]
  pub volume_envelope_divider_period: u8,
  pub constant_volume_envelope: bool,
  pub length_counter_halt: bool,
  #[bits(2)]
  pub duty_cycle: u8,
}

impl APUPulseControlRegister {
  pub fn duty_cycle_float(&self) -> f32 {
    match self.duty_cycle() {
      0 => 0.125,
      1 => 0.25,
      2 => 0.50,
      _ => 0.75,
    }
  }

  pub fn duty_cycle_sequence(&self) -> u8 {
    match self.duty_cycle() {
      0 => 0b00000001,
      1 => 0b00000011,
      2 => 0b00001111,
      _ => 0b11111100,
    }
  }

  pub fn amplitude(&self) -> f32 {
    (self.volume_envelope_divider_period() as f32) / (0b1111 as f32)
  }
}

#[bitfield(u8)]
pub struct APUPulseSweepRegister {
  #[bits(3)]
  pub shift_count: u8,
  pub negate: bool,
  #[bits(3)]
  pub divider_period: u8,
  pub enabled: bool,
}

#[bitfield(u16)]
pub struct APUTimerRegister {
  #[bits(11)]
  pub timer: u16,
  #[bits(5)]
  pub length_counter_load: u8,
}

impl APUTimerRegister {
  pub fn pulse_frequency(&self) -> f32 {
    (NTSC_CPU_FREQUENCY / ((self.timer() as f32 + 1.0) * 16.0)).clamp(0.0, MAX_PULSE_FREQUENCY)
  }

  pub fn triangle_frequency(&self) -> f32 {
    self.pulse_frequency() / 2.0
  }
}

#[bitfield(u8)]
pub struct APUTriangleControlRegister {
  #[bits(7)]
  pub counter_reload_value: u8,
  pub control_flag: bool,
}

#[bitfield(u8)]
pub struct APUNoiseControlRegister {
  #[bits(4)]
  pub volume_envelope_divider_period: u8,
  pub constant_volume_envelope: bool,
  pub length_counter_halt: bool,
  #[bits(2)]
  _unused: u8,
}

#[bitfield(u8)]
pub struct APUNoiseModePeriodRegister {
  #[bits(4)]
  pub period: u8,
  #[bits(3)]
  _unused: u8,
  pub mode: bool,
}

#[bitfield(u8)]
pub struct APUNoiseLengthCounterLoadRegister {
  #[bits(3)]
  _unused: u8,
  #[bits(5)]
  pub length_counter_load: u8,
}

#[bitfield(u8)]
pub struct APUStatusRegister {
  pub pulse1_enable: bool,
  pub pulse2_enable: bool,
  pub triangle_enable: bool,
  pub noise_enable: bool,
  pub dmc_enable: bool,
  pub _unused: bool,
  pub frame_interrupt: bool,
  pub dmc_interrupt: bool,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum APUSequencerMode {
  FourStep = 0,
  FiveStep = 1,
}

impl APUSequencerMode {
  const fn into_bits(self) -> u8 {
    self as _
  }
  const fn from_bits(value: u8) -> Self {
    match value {
      0 => Self::FourStep,
      _ => Self::FiveStep,
    }
  }
}

#[bitfield(u8)]
pub struct APUFrameCounterRegister {
  #[bits(6)]
  _unused: u8,
  pub interrupt_inhibit: bool,
  #[bits(1)]
  pub sequencer_mode: APUSequencerMode,
}
