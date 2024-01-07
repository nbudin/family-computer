use dyn_clone::DynClone;

use self::{cnrom::CNROM, nrom::NROM};
use crate::{bus::BusInterceptor, cpu::CPUBus, ines_rom::INESRom, ppu::PPUMemory};
use std::fmt::Debug;

mod cnrom;
mod nrom;

pub trait CartridgeState {}

#[derive(Debug, Clone, Copy)]
pub enum CartridgeMirroring {
  HORIZONTAL,
  VERTICAL,
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

pub type BoxCartridge = Box<dyn Cartridge + Send + Sync>;

pub fn load_cartridge(rom: INESRom) -> BoxCartridge {
  match rom.mapper_id {
    0 => Box::new(NROM::from_ines_rom(rom)),
    3 => Box::new(CNROM::from_ines_rom(rom)),
    _ => {
      panic!("Unsupported mapper: {}", rom.mapper_id);
    }
  }
}
