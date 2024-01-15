use crate::{
  bus::{Bus, RwHandle},
  cartridge::CartridgeMirroring,
};

use super::PPUMaskRegister;

pub struct PPUMemory<'a> {
  pub mask: PPUMaskRegister,
  pub palette_ram: RwHandle<'a, [u8; 32]>,
  pub name_tables: RwHandle<'a, [[u8; 1024]; 2]>,
  pub pattern_tables: RwHandle<'a, [[u8; 4096]; 2]>,
  pub mirroring: CartridgeMirroring,
}

impl Bus<u16> for PPUMemory<'_> {
  fn try_read_readonly(&self, addr: u16) -> Option<u8> {
    let addr = addr & 0x3fff;

    if addr < 0x1fff {
      Some(self.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff])
    } else if addr < 0x3f00 {
      let addr = addr & 0x0fff;

      match self.mirroring {
        CartridgeMirroring::Horizontal => {
          if addr < 0x0400 {
            Some(self.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0800 {
            Some(self.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0c00 {
            Some(self.name_tables[1][addr as usize & 0x03ff])
          } else {
            Some(self.name_tables[1][addr as usize & 0x03ff])
          }
        }
        CartridgeMirroring::Vertical => {
          if addr < 0x0400 {
            Some(self.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0800 {
            Some(self.name_tables[1][addr as usize & 0x03ff])
          } else if addr < 0x0c00 {
            Some(self.name_tables[0][addr as usize & 0x03ff])
          } else {
            Some(self.name_tables[1][addr as usize & 0x03ff])
          }
        }
        CartridgeMirroring::FourScreen => {
          if addr < 0x400 {
            Some(self.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0800 {
            Some(self.name_tables[1][addr as usize & 0x03ff])
          } else if addr < 0x0c00 {
            Some(self.name_tables[2][addr as usize & 0x03ff])
          } else {
            Some(self.name_tables[3][addr as usize & 0x03ff])
          }
        }
        CartridgeMirroring::SingleScreen => Some(self.name_tables[0][addr as usize & 0x03ff]),
      }
    } else {
      let addr = addr & 0x001f;
      let addr = match addr {
        0x0010 => 0x0000,
        0x0014 => 0x0004,
        0x0018 => 0x0008,
        0x001c => 0x000c,
        _ => addr,
      };
      Some(self.palette_ram[addr as usize] & (if self.mask.grayscale() { 0x30 } else { 0x3f }))
    }
  }

  fn read_side_effects(&mut self, _addr: u16) {}

  fn write(&mut self, addr: u16, value: u8) {
    let addr = addr & 0x3fff;

    if addr < 0x2000 {
      let pattern_tables = self.pattern_tables.try_mut().unwrap();
      pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff] = value;
    } else if addr < 0x3f00 {
      let addr = addr & 0x0fff;
      let name_tables = self.name_tables.try_mut().unwrap();

      match self.mirroring {
        CartridgeMirroring::Horizontal => {
          if addr < 0x0400 {
            name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0800 {
            name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0c00 {
            name_tables[1][addr as usize & 0x03ff] = value;
          } else {
            name_tables[1][addr as usize & 0x03ff] = value;
          }
        }
        CartridgeMirroring::Vertical => {
          if addr < 0x0400 {
            name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0800 {
            name_tables[1][addr as usize & 0x03ff] = value;
          } else if addr < 0x0c00 {
            name_tables[0][addr as usize & 0x03ff] = value;
          } else {
            name_tables[1][addr as usize & 0x03ff] = value;
          }
        }
        CartridgeMirroring::FourScreen => {
          if addr < 0x400 {
            name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0800 {
            name_tables[1][addr as usize & 0x03ff] = value;
          } else if addr < 0x0c00 {
            name_tables[2][addr as usize & 0x03ff] = value;
          } else {
            name_tables[3][addr as usize & 0x03ff] = value;
          }
        }
        CartridgeMirroring::SingleScreen => {
          name_tables[0][addr as usize & 0x03ff] = value;
        }
      }
    } else {
      let addr = addr & 0x001f;
      let addr = match addr {
        0x0010 => 0x0000,
        0x0014 => 0x0004,
        0x0018 => 0x0008,
        0x001c => 0x000c,
        _ => addr,
      };
      let palette_ram = self.palette_ram.try_mut().unwrap();
      palette_ram[addr as usize] = value;
    }
  }
}
