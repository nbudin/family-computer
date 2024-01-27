use dyn_clone::DynClone;

use self::{
  bus_interceptor::BusInterceptor, cnrom::CNROM, mmc1::MMC1, mmc3::MMC3, nrom::NROM, uxrom::UxROM,
};
use crate::{
  apu::APUSynth,
  audio::stream_setup::StreamSpawner,
  cpu::{CPUBus, CPUBusTrait, ExecutedInstruction, CPU},
  nes::INESRom,
  ppu::{PPUCPUBusTrait, PPUMemory, PPUMemoryTrait, Pixbuf, PPU},
};
use std::fmt::Debug;

pub mod bus_interceptor;
mod cnrom;
mod mmc1;
mod mmc3;
mod nrom;
mod uxrom;

#[derive(Debug, Clone, Copy, Default)]
pub enum CartridgeMirroring {
  #[default]
  Horizontal,
  Vertical,
  SingleScreen,
  #[allow(unused)]
  FourScreen,
}

pub trait Mapper: Debug + DynClone {
  type CPUBusInterceptor: BusInterceptor<u16, BusType = CPUBus<Self::PPUMemoryInterceptor>>
    + CPUBusTrait;
  type PPUMemoryInterceptor: BusInterceptor<u16, BusType = PPUMemory> + PPUMemoryTrait;

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

  fn tick_cpu(&mut self, cpu: &mut CPU) -> Option<ExecutedInstruction> {
    cpu.tick(self.cpu_bus_mut())
  }

  fn tick_ppu(&mut self, ppu: &mut PPU, pixbuf: &mut Pixbuf) -> bool {
    ppu.tick(
      pixbuf,
      self.cpu_bus_mut().get_inner_mut().ppu_cpu_bus.as_mut(),
    )
  }

  fn tick_apu(
    &mut self,
    sender: &<APUSynth as StreamSpawner>::OutputType,
    cpu_cycle_count: u64,
  ) -> bool {
    self.cpu_bus_mut().tick_apu(sender, cpu_cycle_count)
  }

  fn poll_irq(&mut self) -> bool {
    false
  }
}

#[macro_export]
macro_rules! memoizing_bus_getters {
  () => {};
}

pub enum Cartridge {
  NROM(Box<NROM>),
  MMC1(Box<MMC1>),
  MMC3(Box<MMC3>),
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
      4 => Cartridge::MMC3(Box::new(MMC3::from_ines_rom(rom))),
      _ => {
        panic!("Unsupported mapper: {}", rom.mapper_id);
      }
    }
  }

  pub fn cpu_bus(&self) -> &dyn CPUBusTrait {
    match self {
      Cartridge::NROM(mapper) => mapper.cpu_bus(),
      Cartridge::MMC1(mapper) => mapper.cpu_bus(),
      Cartridge::MMC3(mapper) => mapper.cpu_bus(),
      Cartridge::UxROM(mapper) => mapper.cpu_bus(),
      Cartridge::CNROM(mapper) => mapper.cpu_bus(),
    }
  }

  pub fn cpu_bus_mut(&mut self) -> &mut dyn CPUBusTrait {
    match self {
      Cartridge::NROM(mapper) => mapper.cpu_bus_mut(),
      Cartridge::MMC1(mapper) => mapper.cpu_bus_mut(),
      Cartridge::MMC3(mapper) => mapper.cpu_bus_mut(),
      Cartridge::UxROM(mapper) => mapper.cpu_bus_mut(),
      Cartridge::CNROM(mapper) => mapper.cpu_bus_mut(),
    }
  }

  pub fn ppu_cpu_bus(&self) -> &dyn PPUCPUBusTrait {
    self.cpu_bus().ppu_cpu_bus()
  }

  pub fn tick_cpu(&mut self, cpu: &mut CPU) -> Option<ExecutedInstruction> {
    match self {
      Cartridge::NROM(mapper) => mapper.tick_cpu(cpu),
      Cartridge::MMC1(mapper) => mapper.tick_cpu(cpu),
      Cartridge::MMC3(mapper) => mapper.tick_cpu(cpu),
      Cartridge::UxROM(mapper) => mapper.tick_cpu(cpu),
      Cartridge::CNROM(mapper) => mapper.tick_cpu(cpu),
    }
  }

  pub fn tick_ppu(&mut self, ppu: &mut PPU, pixbuf: &mut Pixbuf) -> bool {
    match self {
      Cartridge::NROM(mapper) => mapper.tick_ppu(ppu, pixbuf),
      Cartridge::MMC1(mapper) => mapper.tick_ppu(ppu, pixbuf),
      Cartridge::MMC3(mapper) => mapper.tick_ppu(ppu, pixbuf),
      Cartridge::UxROM(mapper) => mapper.tick_ppu(ppu, pixbuf),
      Cartridge::CNROM(mapper) => mapper.tick_ppu(ppu, pixbuf),
    }
  }

  pub fn tick_apu(
    &mut self,
    sender: &<APUSynth as StreamSpawner>::OutputType,
    cpu_cycle_count: u64,
  ) -> bool {
    match self {
      Cartridge::NROM(mapper) => mapper.tick_apu(sender, cpu_cycle_count),
      Cartridge::MMC1(mapper) => mapper.tick_apu(sender, cpu_cycle_count),
      Cartridge::MMC3(mapper) => mapper.tick_apu(sender, cpu_cycle_count),
      Cartridge::UxROM(mapper) => mapper.tick_apu(sender, cpu_cycle_count),
      Cartridge::CNROM(mapper) => mapper.tick_apu(sender, cpu_cycle_count),
    }
  }

  pub fn poll_irq(&mut self) -> bool {
    match self {
      Cartridge::NROM(mapper) => mapper.poll_irq(),
      Cartridge::MMC1(mapper) => mapper.poll_irq(),
      Cartridge::MMC3(mapper) => mapper.poll_irq(),
      Cartridge::UxROM(mapper) => mapper.poll_irq(),
      Cartridge::CNROM(mapper) => mapper.poll_irq(),
    }
  }
}
