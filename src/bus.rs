pub trait Bus<AddrType: Clone> {
  fn try_read_readonly(&self, addr: AddrType) -> Option<u8>;
  fn write(&mut self, addr: AddrType, value: u8);

  fn read_side_effects(&mut self, _addr: AddrType) {}

  fn read_readonly(&self, addr: AddrType) -> u8 {
    Self::try_read_readonly(self, addr).unwrap_or(0)
  }

  fn read(&mut self, addr: AddrType) -> u8 {
    let result = Self::try_read_readonly(&self, addr.clone());
    Self::read_side_effects(self, addr);
    result.unwrap_or(0)
  }
}
