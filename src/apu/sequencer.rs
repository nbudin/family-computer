#[derive(Debug, Clone)]
pub struct APUSequencer {
  pub sequence: u32,
  pub timer: u16,
  pub reload: u16,
  pub output: u8,
}

impl Default for APUSequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl APUSequencer {
  pub fn new() -> Self {
    APUSequencer {
      output: 0,
      reload: 0,
      sequence: 0,
      timer: 0,
    }
  }

  pub fn tick<F: FnOnce(u32) -> u32>(&mut self, enable: bool, f: F) -> u8 {
    if enable {
      self.timer = self.timer.wrapping_sub(1);

      if self.timer == 0xffff {
        self.timer = self.reload + 1;
        self.sequence = f(self.sequence);
        self.output = (self.sequence & 0b1) as u8;
      }
    }

    self.output
  }
}
