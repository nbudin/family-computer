use std::fmt::Debug;

use crate::bus::Bus;

use super::{
  PPUAddressLatch, PPUControlRegister, PPULoopyRegister, PPUMemory, PPUOAMEntry, PPURegister,
  PPUStatusRegister,
};
use crate::cartridge::bus_interceptor::BusInterceptor;

pub struct PPUCPUBus<I: BusInterceptor<u16, BusType = PPUMemory> + ?Sized> {
  pub status: PPUStatusRegister,
  pub control: PPUControlRegister,
  pub data_buffer: u8,
  pub oam: [PPUOAMEntry; 64],
  pub oam_addr: u8,
  pub vram_addr: PPULoopyRegister,
  pub tram_addr: PPULoopyRegister,
  pub fine_x: u8,
  pub address_latch: PPUAddressLatch,
  pub status_register_read_this_tick: bool,
  pub ppu_memory: Box<I>,
}

impl<I: BusInterceptor<u16, BusType = PPUMemory>> Debug for PPUCPUBus<I> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PPUCPUBus")
      .field("status", &self.status)
      .field("control", &self.control)
      .field("data_buffer", &self.data_buffer)
      .field("oam", &self.oam)
      .field("oam_addr", &self.oam_addr)
      .field("vram_addr", &self.vram_addr)
      .field("tram_addr", &self.tram_addr)
      .field("fine_x", &self.fine_x)
      .field(
        "status_register_read_this_tick",
        &self.status_register_read_this_tick,
      )
      .finish_non_exhaustive()
  }
}

impl<I: BusInterceptor<u16, BusType = PPUMemory> + Clone> Clone for PPUCPUBus<I> {
  fn clone(&self) -> Self {
    Self {
      status: self.status,
      control: self.control,
      data_buffer: self.data_buffer,
      oam: self.oam,
      oam_addr: self.oam_addr,
      vram_addr: self.vram_addr,
      tram_addr: self.tram_addr,
      fine_x: self.fine_x,
      address_latch: self.address_latch,
      status_register_read_this_tick: self.status_register_read_this_tick,
      ppu_memory: dyn_clone::clone_box(self.ppu_memory.as_ref()),
    }
  }
}

impl<I: BusInterceptor<u16, BusType = PPUMemory>> PPUCPUBus<I> {
  pub fn new(ppu_memory: Box<I>) -> Self {
    Self {
      status: PPUStatusRegister::from(0),
      control: PPUControlRegister::from(0),
      data_buffer: 0,
      oam: [PPUOAMEntry::new(); 64],
      oam_addr: 0,
      vram_addr: PPULoopyRegister::from(0),
      tram_addr: PPULoopyRegister::from(0),
      fine_x: 0,
      address_latch: PPUAddressLatch::High,
      status_register_read_this_tick: false,
      ppu_memory,
    }
  }
}

impl<I: BusInterceptor<u16, BusType = PPUMemory>> Bus<PPURegister> for PPUCPUBus<I> {
  fn try_read_readonly(&self, addr: PPURegister) -> Option<u8> {
    match addr {
      PPURegister::PPUSTATUS => {
        Some((u8::from(self.status) & 0b11100000) | (self.data_buffer & 0b00011111))
      }
      PPURegister::OAMDATA => {
        let oam_raw: &[u8; 256] = bytemuck::cast_ref(&self.oam);
        Some(oam_raw[self.oam_addr as usize])
      }
      PPURegister::PPUDATA => {
        if u16::from(self.vram_addr) > 0x3f00 {
          self.ppu_memory.try_read_readonly((self.vram_addr).into())
        } else {
          Some(self.data_buffer)
        }
      }
      _ => None,
    }
  }

  fn read_side_effects(&mut self, addr: PPURegister) {
    match addr {
      PPURegister::PPUSTATUS => {
        self.status.set_vertical_blank(false);
        self.address_latch = PPUAddressLatch::High;
        self.status_register_read_this_tick = true;
      }
      PPURegister::PPUDATA => {
        let addr: u16 = (self.vram_addr).into();
        self.data_buffer = self.ppu_memory.read(addr);
        self.vram_addr = PPULoopyRegister::from(
          u16::from(self.vram_addr) + if self.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }
  }

  fn write(&mut self, addr: PPURegister, value: u8) {
    match addr {
      PPURegister::PPUCTRL => {
        self.control = value.into();
        self.tram_addr.set_nametable_x(self.control.nametable_x());
        self.tram_addr.set_nametable_y(self.control.nametable_y());
      }
      PPURegister::PPUMASK => {
        self.ppu_memory.get_inner_mut().mask = value.into();
      }
      PPURegister::OAMADDR => {
        self.oam_addr = value;
      }
      PPURegister::OAMDATA => {
        let oam_raw: &mut [u8; 256] = bytemuck::cast_mut(&mut self.oam);
        oam_raw[self.oam_addr as usize] = value;
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
          self.tram_addr = PPULoopyRegister::from(
            ((u16::from(value) & 0x003f) << 8) | (u16::from(self.tram_addr) & 0x00ff),
          );
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
        let addr: u16 = (self.vram_addr).into();
        self.ppu_memory.write(addr, value);
        self.vram_addr = PPULoopyRegister::from(
          u16::from(self.vram_addr) + if self.control.increment_mode() { 32 } else { 1 },
        );
      }
      _ => {}
    }
  }
}
