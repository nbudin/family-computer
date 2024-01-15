#[derive(Debug, Clone)]
pub struct APULengthCounter {
  pub counter: u8,
  pub enable: bool,
  pub halt: bool,
}

impl Default for APULengthCounter {
    fn default() -> Self {
        Self::new()
    }
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
    } else if self.counter > 0 && !self.halt {
      self.counter -= 1;
    }

    self.counter
  }

  pub fn load_length(&mut self, length_index: u8) {
    self.counter = match length_index {
      0x00 => 10,
      0x01 => 254,
      0x02 => 20,
      0x03 => 2,
      0x04 => 40,
      0x05 => 4,
      0x06 => 80,
      0x07 => 6,
      0x08 => 160,
      0x09 => 8,
      0x0a => 60,
      0x0b => 10,
      0x0c => 14,
      0x0d => 12,
      0x0e => 26,
      0x0f => 14,
      0x10 => 12,
      0x11 => 16,
      0x12 => 24,
      0x13 => 18,
      0x14 => 48,
      0x15 => 20,
      0x16 => 96,
      0x17 => 22,
      0x18 => 192,
      0x19 => 24,
      0x1a => 72,
      0x1b => 26,
      0x1c => 16,
      0x1d => 28,
      0x1e => 32,
      _ => 30,
    };
  }
}
