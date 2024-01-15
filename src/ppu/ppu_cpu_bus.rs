use crate::{
  bus::{Bus, BusInterceptor, RwHandle},
  cartridge::CartridgeMirroring,
};

use super::{
  PPUAddressLatch, PPUControlRegister, PPULoopyRegister, PPUMaskRegister, PPUOAMEntry, PPURegister,
  PPUStatusRegister,
};

pub struct PPUCPUBus<'a> {
  pub status: RwHandle<'a, PPUStatusRegister>,
  pub mask: RwHandle<'a, PPUMaskRegister>,
  pub control: RwHandle<'a, PPUControlRegister>,
  pub data_buffer: RwHandle<'a, u8>,
  pub oam: RwHandle<'a, [PPUOAMEntry; 64]>,
  pub oam_addr: RwHandle<'a, u8>,
  pub vram_addr: RwHandle<'a, PPULoopyRegister>,
  pub tram_addr: RwHandle<'a, PPULoopyRegister>,
  pub fine_x: RwHandle<'a, u8>,
  pub address_latch: RwHandle<'a, PPUAddressLatch>,
  pub status_register_read_this_tick: RwHandle<'a, bool>,
  pub ppu_memory: Box<dyn BusInterceptor<'a, u16> + 'a>,
  pub mirroring: CartridgeMirroring,
}

impl Bus<PPURegister> for PPUCPUBus<'_> {
  fn try_read_readonly(&self, addr: PPURegister) -> Option<u8> {
    match addr {
      PPURegister::PPUSTATUS => {
        Some((u8::from(*self.status) & 0b11100000) | (*self.data_buffer & 0b00011111))
      }
      PPURegister::OAMDATA => {
        let oam_raw: &[u8; 256] = bytemuck::cast_ref(&*self.oam);
        Some(oam_raw[*self.oam_addr as usize])
      }
      PPURegister::PPUDATA => {
        if u16::from(*self.vram_addr) > 0x3f00 {
          self.ppu_memory.try_read_readonly((*self.vram_addr).into())
        } else {
          Some(*self.data_buffer)
        }
      }
      _ => None,
    }
  }

  fn read_side_effects(&mut self, addr: PPURegister) {
    match addr {
      PPURegister::PPUSTATUS => {
        self.status.get_mut().set_vertical_blank(false);
        *self.address_latch.get_mut() = PPUAddressLatch::High;
        *self.status_register_read_this_tick.get_mut() = true;
      }
      PPURegister::PPUDATA => {
        let addr: u16 = (*self.vram_addr).into();
        *self.data_buffer = self.ppu_memory.read(addr);
        *self.vram_addr.get_mut() = PPULoopyRegister::from(
          u16::from(*self.vram_addr) + if self.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }
  }

  fn write(&mut self, addr: PPURegister, value: u8) {
    match addr {
      PPURegister::PPUCTRL => {
        *self.control.get_mut() = value.into();
        self
          .tram_addr
          .get_mut()
          .set_nametable_x(self.control.nametable_x());
        self
          .tram_addr
          .get_mut()
          .set_nametable_y(self.control.nametable_y());
      }
      PPURegister::PPUMASK => {
        *self.mask = value.into();
      }
      PPURegister::OAMADDR => {
        *self.oam_addr = value;
      }
      PPURegister::OAMDATA => {
        let oam_raw: &mut [u8; 256] = bytemuck::cast_mut(&mut *self.oam);
        oam_raw[*self.oam_addr as usize] = value;
      }
      PPURegister::PPUSCROLL => match *self.address_latch {
        PPUAddressLatch::High => {
          *self.fine_x.get_mut() = value & 0x07;
          self.tram_addr.get_mut().set_coarse_x(value >> 3);
          *self.address_latch.get_mut() = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          self.tram_addr.get_mut().set_fine_y(value & 0x07);
          self.tram_addr.get_mut().set_coarse_y(value >> 3);
          *self.address_latch.get_mut() = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUADDR => match *self.address_latch {
        PPUAddressLatch::High => {
          *self.tram_addr.get_mut() = PPULoopyRegister::from(
            ((u16::from(value) & 0x003f) << 8) | (u16::from(*self.tram_addr) & 0x00ff),
          );
          *self.address_latch.get_mut() = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          *self.tram_addr.get_mut() =
            PPULoopyRegister::from((u16::from(*self.tram_addr) & 0xff00) | u16::from(value));
          *self.vram_addr.get_mut() = *self.tram_addr;
          *self.address_latch.get_mut() = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUDATA => {
        let addr: u16 = (*self.vram_addr).into();
        self.ppu_memory.write(addr, value);
        *self.vram_addr.get_mut() = PPULoopyRegister::from(
          u16::from(*self.vram_addr) + if self.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }
  }
}
