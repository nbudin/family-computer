use super::{
  envelope::APUEnvelope, APULengthCounter, APUNoiseControlRegister,
  APUNoiseLengthCounterLoadRegister, APUNoiseModePeriodRegister, APUSequencer,
};

#[derive(Debug, Clone)]
pub struct APUNoiseChannel {
  pub control: APUNoiseControlRegister,
  pub mode_period: APUNoiseModePeriodRegister,
  pub timer_value: u16,
  pub timer_reload: u16,
  pub enabled: bool,
  pub length_counter: APULengthCounter,
  pub envelope: APUEnvelope,
  pub shift_register: APUSequencer,
}

impl Default for APUNoiseChannel {
  fn default() -> Self {
    Self::new()
  }
}

impl APUNoiseChannel {
  pub fn new() -> Self {
    let mut shift_register = APUSequencer::new();
    shift_register.sequence = 1;

    Self {
      control: APUNoiseControlRegister::from(0),
      mode_period: APUNoiseModePeriodRegister::from(0),
      timer_value: 0,
      timer_reload: 0,
      enabled: false,
      length_counter: APULengthCounter::new(),
      envelope: APUEnvelope::new(),
      shift_register,
    }
  }

  pub fn write_control(&mut self, value: APUNoiseControlRegister) {
    self.length_counter.halt = self.control.length_counter_halt();
    self.envelope.enabled = !value.constant_volume_envelope();
    self.envelope.divider = value.volume_envelope_divider_period() as u16;
    self.control = value;
  }

  pub fn write_mode_period(&mut self, value: APUNoiseModePeriodRegister) {
    self.mode_period = value;

    self.timer_reload = match value.period() {
      0 => 4,
      1 => 8,
      2 => 16,
      3 => 32,
      4 => 64,
      5 => 96,
      6 => 128,
      7 => 160,
      8 => 202,
      9 => 254,
      0xa => 380,
      0xb => 508,
      0xc => 762,
      0xd => 1016,
      0xe => 2034,
      _ => 4068,
    }
  }

  pub fn write_length_counter_load(&mut self, value: APUNoiseLengthCounterLoadRegister) {
    self.length_counter.counter = value.length_counter_load();
    // TODO envelope restart
  }
}
