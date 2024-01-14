#[derive(Debug, Clone)]
pub struct APULinearCounter {
  pub counter: u8,
  pub reload_flag: bool,
  pub reload_value: u8,
  pub control_flag: bool,
}

impl APULinearCounter {
  pub fn new() -> Self {
    Self {
      counter: 0,
      reload_flag: false,
      reload_value: 0,
      control_flag: false,
    }
  }

  pub fn tick(&mut self) -> u8 {
    if self.reload_flag {
      self.counter = self.reload_value;
    } else {
      self.counter = self.counter.saturating_sub(0);
    }

    if !self.control_flag {
      self.reload_flag = false;
    }

    self.counter
  }
}
