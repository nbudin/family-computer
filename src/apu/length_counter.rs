#[derive(Debug, Clone)]
pub struct APULengthCounter {
  pub counter: u8,
  pub enable: bool,
  pub halt: bool,
}

impl APULengthCounter {
  pub fn new() -> Self {
    Self {
      counter: 0,
      enable: true,
      halt: false,
    }
  }

  pub fn tick(&mut self) -> u8 {
    if !self.enable {
      self.counter = 0;
    } else {
      if self.counter > 0 && !self.halt {
        self.counter -= 1;
      }
    }

    self.counter
  }
}
