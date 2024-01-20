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
pub struct CNROMCPUBusInterceptor {
  prg_rom: [u8; 32 * 1024],
  bus: CPUBus<CNROMPPUMemoryInterceptor>,
}

impl BusInterceptor<u16> for CNROMCPUBusInterceptor {
  type BusType = CPUBus<CNROMPPUMemoryInterceptor>;

  fn get_inner(&self) -> &CPUBus<CNROMPPUMemoryInterceptor> {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut CPUBus<CNROMPPUMemoryInterceptor> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else {
      InterceptorResult::Intercepted(Some(self.prg_rom[usize::from(addr - 0x8000)]))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else {
      self.bus.ppu_cpu_bus.ppu_memory.bank_select = value & 0b11;
      InterceptorResult::Intercepted(())
    }
  }
}

#[derive(Debug, Clone)]
pub struct CNROMPPUMemoryInterceptor {
  bank_select: u8,
  chr_rom: [u8; 4 * 8 * 1024],
  bus: PPUMemory,
}

impl BusInterceptor<u16> for CNROMPPUMemoryInterceptor {
  type BusType = PPUMemory;

  fn get_inner(&self) -> &PPUMemory {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut PPUMemory {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x2000 {
      InterceptorResult::Intercepted(Some(
        self.chr_rom[(self.bank_select as usize * 8 * 1024) + usize::from(addr)],
      ))
    } else {
      InterceptorResult::NotIntercepted
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x2000 {
      let addr = (self.bank_select as usize * 8 * 1024) + usize::from(addr);
      self.chr_rom[addr] = value;
      InterceptorResult::Intercepted(())
    } else {
      InterceptorResult::NotIntercepted
    }
  }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct CNROM {
  cpu_bus: CNROMCPUBusInterceptor,
}

impl Mapper for CNROM {
  type CPUBusInterceptor = CNROMCPUBusInterceptor;
  type PPUMemoryInterceptor = CNROMPPUMemoryInterceptor;

  fn from_ines_rom(rom: INESRom) -> Self {
    let mut prg_rom: [u8; 32 * 1024] = [0; 32 * 1024];
    if !rom.prg_data.is_empty() {
      for chunk in prg_rom.chunks_exact_mut(rom.prg_data.len()) {
        chunk.copy_from_slice(&rom.prg_data);
      }
    }

    let mut chr_rom: [u8; 4 * 8 * 1024] = [0; 4 * 8 * 1024];
    if !rom.chr_data.is_empty() {
      for chunk in chr_rom.chunks_exact_mut(rom.chr_data.len()) {
        chunk.copy_from_slice(&rom.chr_data);
      }
    }

    let ppu_memory = CNROMPPUMemoryInterceptor {
      bank_select: 0,
      chr_rom,
      bus: PPUMemory::new(rom.initial_mirroring()),
    };

    let cpu_bus = CNROMCPUBusInterceptor {
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
