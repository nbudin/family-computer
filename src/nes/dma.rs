use crate::ppu::PPUOAMEntry;

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct DMA {
  pub page: u8,
  pub addr: u8,
  pub data: u8,
  pub transfer: bool,
  pub dummy: bool,
}

impl DMA {
  pub fn new() -> Self {
    Self {
      page: 0,
      addr: 0,
      data: 0,
      transfer: false,
      dummy: true,
    }
  }

  pub fn ram_addr(&self) -> u16 {
    (self.page as u16) << 8 | (self.addr as u16)
  }

  pub fn store_data(&mut self, value: u8) {
    self.data = value;
  }

  pub fn write_to_ppu(&mut self, oam: &mut [PPUOAMEntry; 64]) {
    let oam_raw: &mut [u8; 256] = bytemuck::cast_mut(oam);
    oam_raw[self.addr as usize] = self.data;
    self.addr = self.addr.wrapping_add(1);

    if self.addr == 0 {
      self.transfer = false;
      self.dummy = true;
    }
  }
}

impl Default for DMA {
  fn default() -> Self {
    Self::new()
  }
}
