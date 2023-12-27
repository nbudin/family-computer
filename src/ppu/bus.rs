use crate::machine::Machine;

use super::{PPUAddressLatch, PPULoopyRegister, PPURegister, PPU};

impl PPU {
  pub fn read_bus(mut self, machine: &Machine, register: PPURegister) -> (Self, u8) {
    let mut result: u8 = 0;

    match register {
      PPURegister::PPUSTATUS => {
        result = (u8::from(self.status) & 0b11100000) | (self.data_buffer & 0b00011111);
        self.status.set_vertical_blank(false);
        self.address_latch = PPUAddressLatch::High;
      }
      PPURegister::PPUDATA => {
        result = self.data_buffer;
        self.data_buffer = self.get_ppu_mem(machine, self.vram_addr.into());

        if u16::from(self.vram_addr) > 0x3f00 {
          // palette memory is read immediately
          result = self.data_buffer;
        }

        self.vram_addr = PPULoopyRegister::from(
          u16::from(self.vram_addr) + if self.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }

    (self, result)
  }

  pub fn write_bus(mut self, machine: &mut Machine, register: PPURegister, value: u8) -> Self {
    match register {
      PPURegister::PPUCTRL => {
        self.control = value.into();
        self.tram_addr.set_nametable_x(self.control.nametable_x());
        self.tram_addr.set_nametable_y(self.control.nametable_y());
      }
      PPURegister::PPUMASK => {
        self.mask = value.into();
      }
      PPURegister::PPUSCROLL => match self.address_latch {
        PPUAddressLatch::High => {
          self.fine_x = value & 0x07;
          self.tram_addr.set_coarse_x(value >> 3);
          self.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          self.tram_addr.set_fine_y(value & 0x07);
          self.tram_addr.set_coarse_y(value >> 3);
          self.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUADDR => match self.address_latch {
        PPUAddressLatch::High => {
          self.tram_addr =
            PPULoopyRegister::from((u16::from(self.tram_addr) & 0x00ff) | (u16::from(value) << 8));
          self.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          self.tram_addr =
            PPULoopyRegister::from((u16::from(self.tram_addr) & 0xff00) | u16::from(value));
          self.vram_addr = self.tram_addr;
          self.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUDATA => {
        self.set_ppu_mem(machine, u16::from(self.vram_addr), value);
        self.vram_addr = PPULoopyRegister::from(
          u16::from(self.vram_addr) + if self.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }

    self
  }
}
