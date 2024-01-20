use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::{bus::Bus, cartridge::CartridgeMirroring};

use super::PPUMaskRegister;

pub trait GetMirroringFn: DynClone + Send + Sync {
  fn get_mirroring(&self) -> CartridgeMirroring;
}

impl<F: Fn() -> CartridgeMirroring> GetMirroringFn for F
where
  F: DynClone + Send + Sync,
{
  fn get_mirroring(&self) -> CartridgeMirroring {
    (self)()
  }
}

pub trait PPUMemoryTrait: Bus<u16> {
  fn mask(&self) -> PPUMaskRegister;
}

#[derive(Debug, Clone)]
pub struct PPUMemory {
  pub mask: PPUMaskRegister,
  pub palette_ram: [u8; 32],
  pub name_tables: [[u8; 1024]; 4],
  pub pattern_tables: [[u8; 4096]; 2],
  pub mirroring: CartridgeMirroring,
}

impl PPUMemory {
  pub fn new(mirroring: CartridgeMirroring) -> Self {
    Self {
      mask: PPUMaskRegister::from(0),
      mirroring,
      name_tables: [[0; 1024]; 4],
      pattern_tables: [[0; 4096]; 2],
      palette_ram: [0; 32],
    }
  }
}

impl PPUMemoryTrait for PPUMemory {
  fn mask(&self) -> PPUMaskRegister {
    self.mask.clone()
  }
}

impl Bus<u16> for PPUMemory {
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
      self.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff] = value;
    } else if addr < 0x3f00 {
      let addr = addr & 0x0fff;
      let name_tables = &mut self.name_tables;

      match self.mirroring {
        CartridgeMirroring::Horizontal => {
          if addr < 0x0800 {
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
      let palette_ram = &mut self.palette_ram;
      palette_ram[addr as usize] = value;
    }
  }
}
