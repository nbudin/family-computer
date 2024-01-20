use dyn_clone::DynClone;

use self::{bus_interceptor::BusInterceptor, cnrom::CNROM, mmc1::MMC1, nrom::NROM, uxrom::UxROM};
use crate::{
  bus::Bus,
  cpu::{CPUBus, CPUBusTrait},
  nes::INESRom,
  ppu::PPUMemory,
};
use std::fmt::Debug;

pub mod bus_interceptor;
mod cnrom;
mod mmc1;
mod nrom;
mod uxrom;

#[derive(Debug, Clone, Copy)]
pub enum CartridgeMirroring {
  Horizontal,
  Vertical,
  SingleScreen,
  #[allow(unused)]
  FourScreen,
}

pub trait Mapper: Debug + DynClone {
  type CPUBusInterceptor: BusInterceptor<u16, BusType = CPUBus<Self::PPUMemoryInterceptor>>;
  type PPUMemoryInterceptor: BusInterceptor<u16, BusType = PPUMemory>;

  fn from_ines_rom(rom: INESRom) -> Self
  where
    Self: Sized;

  fn cpu_bus(&self) -> &Self::CPUBusInterceptor;
  fn cpu_bus_mut(&mut self) -> &mut Self::CPUBusInterceptor;

  fn ppu_memory(&self) -> &Self::PPUMemoryInterceptor {
    self.cpu_bus().get_inner().ppu_cpu_bus.ppu_memory.as_ref()
  }

  fn ppu_memory_mut(&mut self) -> &mut Self::PPUMemoryInterceptor {
    self
      .cpu_bus_mut()
      .get_inner_mut()
      .ppu_cpu_bus
      .ppu_memory
      .as_mut()
  }
}

#[macro_export]
macro_rules! memoizing_bus_getters {
  () => {};
}

pub enum Cartridge {
  NROM(Box<NROM>),
  MMC1(Box<MMC1>),
  UxROM(Box<UxROM>),
  CNROM(Box<CNROM>),
}

impl Cartridge {
  pub fn from_ines_rom(rom: INESRom) -> Self {
    match rom.mapper_id {
      0 => Cartridge::NROM(Box::new(NROM::from_ines_rom(rom))),
      1 => Cartridge::MMC1(Box::new(MMC1::from_ines_rom(rom))),
      2 => Cartridge::UxROM(Box::new(UxROM::from_ines_rom(rom))),
      3 => Cartridge::CNROM(Box::new(CNROM::from_ines_rom(rom))),
      _ => {
        panic!("Unsupported mapper: {}", rom.mapper_id);
      }
    }
  }

  pub fn cpu_bus(&self) -> &dyn CPUBusTrait {
    match self {
      Cartridge::NROM(mapper) => mapper.cpu_bus(),
      Cartridge::MMC1(mapper) => mapper.cpu_bus(),
      Cartridge::UxROM(mapper) => mapper.cpu_bus(),
      Cartridge::CNROM(mapper) => mapper.cpu_bus(),
    }
  }

  pub fn cpu_bus_mut(&mut self) -> &mut dyn CPUBusTrait {
    match self {
      Cartridge::NROM(mapper) => mapper.cpu_bus_mut(),
      Cartridge::MMC1(mapper) => mapper.cpu_bus_mut(),
      Cartridge::UxROM(mapper) => mapper.cpu_bus_mut(),
      Cartridge::CNROM(mapper) => mapper.cpu_bus_mut(),
    }
  }

  #[allow(unused)]
  pub fn ppu_memory(&self) -> &dyn Bus<u16> {
    match self {
      Cartridge::NROM(mapper) => mapper.ppu_memory(),
      Cartridge::MMC1(mapper) => mapper.ppu_memory(),
      Cartridge::UxROM(mapper) => mapper.ppu_memory(),
      Cartridge::CNROM(mapper) => mapper.ppu_memory(),
    }
  }

  pub fn ppu_memory_mut(&mut self) -> &mut dyn Bus<u16> {
    match self {
      Cartridge::NROM(mapper) => mapper.ppu_memory_mut(),
      Cartridge::MMC1(mapper) => mapper.ppu_memory_mut(),
      Cartridge::UxROM(mapper) => mapper.ppu_memory_mut(),
      Cartridge::CNROM(mapper) => mapper.ppu_memory_mut(),
    }
  }
}
