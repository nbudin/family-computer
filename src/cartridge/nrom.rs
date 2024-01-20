use crate::{
  cpu::CPUBus,
  nes::INESRom,
  ppu::{PPUCPUBus, PPUMemory},
};

use super::{
  bus_interceptor::{BusInterceptor, InterceptorResult},
  Mapper,
};

#[derive(Debug, Clone)]
pub struct NROMCPUBusInterceptor {
  prg_ram: [u8; 8 * 1024],
  prg_rom: [u8; 32 * 1024],
  bus: CPUBus<NROMPPUMemoryInterceptor>,
}

impl BusInterceptor<u16> for NROMCPUBusInterceptor {
  type BusType = CPUBus<NROMPPUMemoryInterceptor>;

  fn get_inner(&self) -> &CPUBus<NROMPPUMemoryInterceptor> {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut CPUBus<NROMPPUMemoryInterceptor> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      InterceptorResult::Intercepted(Some(self.prg_ram[usize::from(addr) % (8 * 1024)]))
    } else {
      InterceptorResult::Intercepted(Some(self.prg_rom[usize::from(addr - 0x8000)]))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      self.prg_ram[usize::from(addr) % (8 * 1024)] = value;
      InterceptorResult::Intercepted(())
    } else {
      // can't write to rom
      InterceptorResult::Intercepted(())
    }
  }
}

#[derive(Debug, Clone)]
pub struct NROMPPUMemoryInterceptor {
  chr_rom: [u8; 8 * 1024],
  bus: PPUMemory,
}

impl BusInterceptor<u16> for NROMPPUMemoryInterceptor {
  type BusType = PPUMemory;

  fn get_inner(&self) -> &PPUMemory {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut PPUMemory {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x2000 {
      InterceptorResult::Intercepted(Some(self.chr_rom[usize::from(addr)]))
    } else {
      InterceptorResult::NotIntercepted
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x2000 {
      self.chr_rom[usize::from(addr)] = value;
      InterceptorResult::Intercepted(())
    } else {
      InterceptorResult::NotIntercepted
    }
  }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NROM {
  cpu_bus: NROMCPUBusInterceptor,
}

impl Mapper for NROM {
  type CPUBusInterceptor = NROMCPUBusInterceptor;
  type PPUMemoryInterceptor = NROMPPUMemoryInterceptor;

  fn from_ines_rom(rom: INESRom) -> Self {
    let mut prg_rom: [u8; 32 * 1024] = [0; 32 * 1024];
    if !rom.prg_data.is_empty() {
      for chunk in prg_rom.chunks_exact_mut(rom.prg_data.len()) {
        chunk.copy_from_slice(&rom.prg_data);
      }
    }

    let mut chr_rom: [u8; 8 * 1024] = [0; 8 * 1024];
    if !rom.chr_data.is_empty() {
      for chunk in chr_rom.chunks_exact_mut(rom.chr_data.len()) {
        chunk.copy_from_slice(&rom.chr_data);
      }
    }

    let ppu_memory = NROMPPUMemoryInterceptor {
      chr_rom,
      bus: PPUMemory::new(rom.initial_mirroring()),
    };

    let cpu_bus = NROMCPUBusInterceptor {
      prg_ram: [0; 8 * 1024],
      prg_rom,
      bus: CPUBus::new(PPUCPUBus::new(Box::new(ppu_memory))),
    };

    Self { cpu_bus }
  }

  fn cpu_bus(&self) -> &Self::CPUBusInterceptor {
    &self.cpu_bus
  }

  fn cpu_bus_mut(&mut self) -> &mut Self::CPUBusInterceptor {
    &mut self.cpu_bus
  }
}
