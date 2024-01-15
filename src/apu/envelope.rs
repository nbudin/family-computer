#[derive(Debug, Clone)]
pub struct APUEnvelope {
  pub start_flag: bool,
  pub loop_flag: bool,
  pub enabled: bool,
  pub divider: u16,
  pub decay: u16,
  pub volume: u16,
  pub output: u16,
}

impl Default for APUEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

impl APUEnvelope {
  pub fn new() -> Self {
    Self {
      start_flag: false,
      loop_flag: false,
      enabled: true,
      divider: 0,
      decay: 0,
      volume: 0,
      output: 0,
    }
  }

  pub fn tick(&mut self) {
    if !self.start_flag {
      if self.divider == 0 {
        self.divider = self.volume;

        if self.decay == 0 {
          if self.loop_flag {
            self.decay = 15;
          }
        } else {
          self.decay -= 1;
        }
      } else {
        self.divider -= 1;
      }
    } else {
      self.start_flag = false;
      self.decay = 15;
      self.divider = self.volume;
    }

    self.output = if self.enabled {
      self.decay
    } else {
      self.volume
    };
  }
}
