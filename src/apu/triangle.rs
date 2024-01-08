use super::registers::{APUTimerRegister, APUTriangleControlRegister};

#[derive(Debug, Clone)]
pub struct APUTriangleChannel {
  pub control: APUTriangleControlRegister,
  pub timer: APUTimerRegister,
}

impl APUTriangleChannel {
  pub fn new() -> Self {
    Self {
      control: 0.into(),
      timer: 0.into(),
    }
  }

  pub fn write_control(&mut self, value: APUTriangleControlRegister) {
    self.control = value;
  }

  pub fn write_timer_byte(&mut self, value: u8, high_byte: bool) {
    let new_value = if high_byte {
      APUTimerRegister::from((u16::from(self.timer) & 0x00ff) | ((value as u16) << 8))
    } else {
      APUTimerRegister::from((u16::from(self.timer) & 0xff00) | (value as u16))
    };

    self.timer = new_value;
  }
}
