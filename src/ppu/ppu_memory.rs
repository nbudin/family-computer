use crate::{bus::Bus, cartridge::CartridgeMirroring, rw_handle::RwHandle};

use super::PPU;

pub struct PPUMemory<'a> {
  pub ppu: RwHandle<'a, PPU>,
  pub mirroring: CartridgeMirroring,
}

impl Bus<u16> for PPUMemory<'_> {
  fn try_read_readonly(&self, addr: u16) -> Option<u8> {
    let addr = addr & 0x3fff;

    if addr < 0x1fff {
      Some(self.ppu.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff])
    } else if addr < 0x3f00 {
      let addr = addr & 0x0fff;

      match self.mirroring {
        CartridgeMirroring::HORIZONTAL => {
          if addr < 0x0400 {
            Some(self.ppu.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0800 {
            Some(self.ppu.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0c00 {
            Some(self.ppu.name_tables[1][addr as usize & 0x03ff])
          } else {
            Some(self.ppu.name_tables[1][addr as usize & 0x03ff])
          }
        }
        CartridgeMirroring::VERTICAL => {
          if addr < 0x0400 {
            Some(self.ppu.name_tables[0][addr as usize & 0x03ff])
          } else if addr < 0x0800 {
            Some(self.ppu.name_tables[1][addr as usize & 0x03ff])
          } else if addr < 0x0c00 {
            Some(self.ppu.name_tables[0][addr as usize & 0x03ff])
          } else {
            Some(self.ppu.name_tables[1][addr as usize & 0x03ff])
          }
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
      Some(
        self.ppu.palette_ram[addr as usize]
          & (if self.ppu.mask.grayscale() {
            0x30
          } else {
            0x3f
          }),
      )
    }
  }

  fn read_side_effects(&mut self, _addr: u16) {}

  fn write(&mut self, addr: u16, value: u8) {
    let addr = addr & 0x3fff;

    let ppu = self.ppu.try_mut().unwrap();

    if addr < 0x2000 {
      ppu.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff] = value;
    } else if addr < 0x3f00 {
      let addr = addr & 0x0fff;

      match self.mirroring {
        CartridgeMirroring::HORIZONTAL => {
          if addr < 0x0400 {
            ppu.name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0800 {
            ppu.name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0c00 {
            ppu.name_tables[1][addr as usize & 0x03ff] = value;
          } else {
            ppu.name_tables[1][addr as usize & 0x03ff] = value;
          }
        }
        CartridgeMirroring::VERTICAL => {
          if addr < 0x0400 {
            ppu.name_tables[0][addr as usize & 0x03ff] = value;
          } else if addr < 0x0800 {
            ppu.name_tables[1][addr as usize & 0x03ff] = value;
          } else if addr < 0x0c00 {
            ppu.name_tables[0][addr as usize & 0x03ff] = value;
          } else {
            ppu.name_tables[1][addr as usize & 0x03ff] = value;
          }
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
      ppu.palette_ram[addr as usize] = value;
    }
  }
}
