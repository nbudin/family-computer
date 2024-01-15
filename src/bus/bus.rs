pub trait Bus<AddrType: Clone> {
  fn try_read_readonly(&self, addr: AddrType) -> Option<u8>;
  fn write(&mut self, addr: AddrType, value: u8);

  fn read_side_effects(&mut self, _addr: AddrType) {}

  fn read_readonly(&self, addr: AddrType) -> u8 {
    Self::try_read_readonly(self, addr).unwrap_or(0)
  }

  fn read(&mut self, addr: AddrType) -> u8 {
    let result = Self::try_read_readonly(self, addr.clone());
    Self::read_side_effects(self, addr);
    result.unwrap_or(0)
  }
}

#[cfg(test)]
mod tests {
  pub use super::*;

  struct FakeBus {
    pub memory: [u8; 256],
    pub address_42_read: bool,
  }

  impl FakeBus {
    pub fn new() -> Self {
      Self {
        memory: [0; 256],
        address_42_read: false,
      }
    }
  }

  impl Bus<usize> for FakeBus {
    fn try_read_readonly(&self, addr: usize) -> Option<u8> {
      if addr < self.memory.len() {
        Some(self.memory[addr])
      } else {
        None
      }
    }

    fn read_side_effects(&mut self, addr: usize) {
      if addr == 42 {
        self.address_42_read = true;
      }
    }

    fn write(&mut self, addr: usize, value: u8) {
      if addr < self.memory.len() {
        self.memory[addr] = value;
      }
    }
  }

  #[test]
  fn test_read_readonly() {
    let mut bus = FakeBus::new();
    bus.memory[41] = 1;
    bus.memory[42] = 2;

    assert_eq!(bus.read_readonly(41), 1);
    assert_eq!(bus.read_readonly(42), 2);
    assert_eq!(bus.read_readonly(512), 0);
    assert!(!bus.address_42_read);
  }

  #[test]
  fn test_read() {
    let mut bus = FakeBus::new();
    bus.memory[41] = 1;
    bus.memory[42] = 2;

    assert_eq!(bus.read(41), 1);
    assert!(!bus.address_42_read);
    assert_eq!(bus.read(42), 2);
    assert_eq!(bus.read(512), 0);
    assert!(bus.address_42_read);
  }

  #[test]
  fn test_write() {
    let mut bus = FakeBus::new();
    bus.write(41, 1);
    bus.write(42, 2);
    bus.write(512, 3);

    assert_eq!(bus.memory[41], 1);
    assert_eq!(bus.memory[42], 2);
  }
}
