use crate::{cartridge::CartridgeMirroring, machine::Machine};

use super::PPU;

impl PPU {
  pub fn get_ppu_mem(&self, machine: &Machine, addr: u16) -> u8 {
    let addr = addr & 0x3fff;

    let cartridge = &machine.cartridge;

    match cartridge.get_ppu_mem(addr) {
      Some(value) => value,
      None => {
        if addr < 0x1fff {
          self.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff]
        } else if addr < 0x3f00 {
          let addr = addr & 0x0fff;

          match cartridge.get_mirroring() {
            CartridgeMirroring::HORIZONTAL => {
              if addr < 0x0400 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else if addr < 0x0800 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else if addr < 0x0c00 {
                self.name_tables[1][addr as usize & 0x03ff]
              } else {
                self.name_tables[1][addr as usize & 0x03ff]
              }
            }
            CartridgeMirroring::VERTICAL => {
              if addr < 0x0400 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else if addr < 0x0800 {
                self.name_tables[1][addr as usize & 0x03ff]
              } else if addr < 0x0c00 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else {
                self.name_tables[1][addr as usize & 0x03ff]
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
          self.palette_ram[addr as usize] & (if self.mask.grayscale() { 0x30 } else { 0x3f })
        }
      }
    }
  }

  pub fn set_ppu_mem(machine: &mut Machine, addr: u16, value: u8) {
    let addr = addr & 0x3fff;

    let cartridge = &mut machine.cartridge;

    if cartridge.set_ppu_mem(addr, value) {
    } else {
      if addr < 0x2000 {
        machine.ppu.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff] = value;
      } else if addr < 0x3f00 {
        let addr = addr & 0x0fff;

        match cartridge.get_mirroring() {
          CartridgeMirroring::HORIZONTAL => {
            if addr < 0x0400 {
              machine.ppu.name_tables[0][addr as usize & 0x03ff] = value;
            } else if addr < 0x0800 {
              machine.ppu.name_tables[0][addr as usize & 0x03ff] = value;
            } else if addr < 0x0c00 {
              machine.ppu.name_tables[1][addr as usize & 0x03ff] = value;
            } else {
              machine.ppu.name_tables[1][addr as usize & 0x03ff] = value;
            }
          }
          CartridgeMirroring::VERTICAL => {
            if addr < 0x0400 {
              machine.ppu.name_tables[0][addr as usize & 0x03ff] = value;
            } else if addr < 0x0800 {
              machine.ppu.name_tables[1][addr as usize & 0x03ff] = value;
            } else if addr < 0x0c00 {
              machine.ppu.name_tables[0][addr as usize & 0x03ff] = value;
            } else {
              machine.ppu.name_tables[1][addr as usize & 0x03ff] = value;
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
        machine.ppu.palette_ram[addr as usize] = value;
      }
    }
  }
}
