use crate::{bus::Bus, cartridge::CartridgeMirroring, rw_handle::RwHandle};

use super::{PPUAddressLatch, PPULoopyRegister, PPUMemory, PPURegister, PPU};

pub struct PPUCPUBus<'a> {
  pub ppu: RwHandle<'a, PPU>,
  pub mirroring: CartridgeMirroring,
}

impl Bus<PPURegister> for PPUCPUBus<'_> {
  fn try_read_readonly(&self, addr: PPURegister) -> Option<u8> {
    match addr {
      PPURegister::PPUSTATUS => {
        Some((u8::from(self.ppu.status) & 0b11100000) | (self.ppu.data_buffer & 0b00011111))
      }
      PPURegister::OAMDATA => {
        let oam_raw: &[u8; 256] = bytemuck::cast_ref(&self.ppu.oam);
        Some(oam_raw[self.ppu.oam_addr as usize])
      }
      PPURegister::PPUDATA => {
        if u16::from(self.ppu.vram_addr) > 0x3f00 {
          // palette memory is read immediately
          let ppu_memory = PPUMemory {
            mirroring: self.mirroring,
            ppu: RwHandle::ReadOnly(&self.ppu),
          };
          ppu_memory.try_read_readonly(self.ppu.vram_addr.into())
        } else {
          Some(self.ppu.data_buffer)
        }
      }
      _ => None,
    }
  }

  fn read_side_effects(&mut self, addr: PPURegister) {
    let ppu = self.ppu.try_mut().unwrap();

    match addr {
      PPURegister::PPUSTATUS => {
        ppu.status.set_vertical_blank(false);
        ppu.address_latch = PPUAddressLatch::High;
        ppu.status_register_read_this_tick = true;
      }
      PPURegister::PPUDATA => {
        let addr: u16 = ppu.vram_addr.into();
        let mut ppu_memory = PPUMemory {
          ppu: RwHandle::ReadWrite(ppu),
          mirroring: self.mirroring,
        };
        ppu.data_buffer = ppu_memory.read(addr);
        ppu.vram_addr = PPULoopyRegister::from(
          u16::from(ppu.vram_addr) + if ppu.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }
  }

  fn write(&mut self, addr: PPURegister, value: u8) {
    let ppu = self.ppu.try_mut().unwrap();

    match addr {
      PPURegister::PPUCTRL => {
        ppu.control = value.into();
        ppu.tram_addr.set_nametable_x(ppu.control.nametable_x());
        ppu.tram_addr.set_nametable_y(ppu.control.nametable_y());
      }
      PPURegister::PPUMASK => {
        ppu.mask = value.into();
      }
      PPURegister::OAMADDR => {
        ppu.oam_addr = value;
      }
      PPURegister::OAMDATA => {
        let oam_raw: &mut [u8; 256] = bytemuck::cast_mut(&mut ppu.oam);
        oam_raw[ppu.oam_addr as usize] = value;
      }
      PPURegister::PPUSCROLL => match ppu.address_latch {
        PPUAddressLatch::High => {
          ppu.fine_x = value & 0x07;
          ppu.tram_addr.set_coarse_x(value >> 3);
          ppu.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          ppu.tram_addr.set_fine_y(value & 0x07);
          ppu.tram_addr.set_coarse_y(value >> 3);
          ppu.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUADDR => match ppu.address_latch {
        PPUAddressLatch::High => {
          ppu.tram_addr = PPULoopyRegister::from(
            ((u16::from(value) & 0x003f) << 8) | (u16::from(ppu.tram_addr) & 0x00ff),
          );
          ppu.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          ppu.tram_addr =
            PPULoopyRegister::from((u16::from(ppu.tram_addr) & 0xff00) | u16::from(value));
          ppu.vram_addr = ppu.tram_addr;
          ppu.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUDATA => {
        let addr: u16 = ppu.vram_addr.into();
        let mut ppu_memory = PPUMemory {
          ppu: RwHandle::ReadWrite(ppu),
          mirroring: self.mirroring,
        };
        ppu_memory.write(addr, value);
        ppu.vram_addr = PPULoopyRegister::from(
          u16::from(ppu.vram_addr) + if ppu.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }
  }
}
