use dyn_clone::DynClone;

use self::{cnrom::CNROM, nrom::NROM};
use crate::{bus::Bus, cpu::CPUBus, ines_rom::INESRom, ppu::PPUMemory};
use std::fmt::Debug;

mod cnrom;
mod nrom;

pub trait CartridgeState {}

#[derive(Debug, Clone, Copy)]
pub enum CartridgeMirroring {
  HORIZONTAL,
  VERTICAL,
}

pub enum InterceptorResult<T> {
  Intercepted(T),
  NotIntercepted,
}

pub trait BusInterceptor<'a, AddrType: Clone> {
  fn bus(&self) -> &dyn Bus<AddrType>;
  fn bus_mut(&mut self) -> &mut dyn Bus<AddrType>;

  fn intercept_read_readonly(&self, addr: AddrType) -> InterceptorResult<Option<u8>>;
  fn intercept_write(&mut self, addr: AddrType, value: u8) -> InterceptorResult<()>;
  fn intercept_read_side_effects(&mut self, _addr: AddrType) {}
}

impl<'a, AddrType: Clone, I: BusInterceptor<'a, AddrType> + ?Sized> Bus<AddrType> for I {
  fn try_read_readonly(&self, addr: AddrType) -> Option<u8> {
    match self.intercept_read_readonly(addr.clone()) {
      InterceptorResult::Intercepted(value) => value,
      InterceptorResult::NotIntercepted => self.bus().try_read_readonly(addr),
    }
  }

  fn read_side_effects(&mut self, addr: AddrType) {
    self.intercept_read_side_effects(addr);
  }

  fn write(&mut self, addr: AddrType, value: u8) {
    match self.intercept_write(addr.clone(), value) {
      InterceptorResult::Intercepted(_) => {}
      InterceptorResult::NotIntercepted => self.bus_mut().write(addr, value),
    }
  }
}

pub trait Cartridge: Debug + DynClone {
  fn from_ines_rom(rom: INESRom) -> Self
  where
    Self: Sized;

  fn cpu_bus_interceptor<'a>(&'a self, bus: CPUBus<'a>) -> Box<dyn BusInterceptor<'a, u16> + 'a>;
  fn cpu_bus_interceptor_mut<'a>(
    &'a mut self,
    bus: CPUBus<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a>;
  fn ppu_memory_interceptor<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a>;
  fn ppu_memory_interceptor_mut<'a>(
    &'a mut self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a>;
  fn get_mirroring(&self) -> CartridgeMirroring;
}

pub type BoxCartridge = Box<dyn Cartridge>;

pub fn load_cartridge(rom: INESRom) -> BoxCartridge {
  match rom.mapper_id {
    0 => Box::new(NROM::from_ines_rom(rom)),
    3 => Box::new(CNROM::from_ines_rom(rom)),
    _ => {
      panic!("Unsupported mapper: {}", rom.mapper_id);
    }
  }
}
