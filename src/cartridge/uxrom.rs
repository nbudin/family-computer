use crate::{
  cpu::CPUBus,
  ppu::{PPUCPUBus, PPUMemory},
};

use super::{
  bus_interceptor::{BusInterceptor, InterceptorResult},
  Mapper,
};

#[derive(Debug, Clone)]
pub struct UxROMCPUBusInterceptor {
  prg_rom: Vec<u8>,
  bank_select: u8,
  bus: CPUBus<UxROMPPUMemoryInterceptor>,
}

impl BusInterceptor<u16> for UxROMCPUBusInterceptor {
  type BusType = CPUBus<UxROMPPUMemoryInterceptor>;

  fn get_inner(&self) -> &CPUBus<UxROMPPUMemoryInterceptor> {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut CPUBus<UxROMPPUMemoryInterceptor> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0xc000 {
      InterceptorResult::Intercepted(Some(
        *self
          .prg_rom
          .get(
            ((0x4000 * (self.bank_select as usize)) + usize::from(addr - 0x8000))
              % self.prg_rom.len(),
          )
          .unwrap(),
      ))
    } else {
      InterceptorResult::Intercepted(Some(
        self.prg_rom[self.prg_rom.len() - 0x4000 + usize::from(addr - 0xc000)],
      ))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else {
      self.bank_select = value;
      InterceptorResult::Intercepted(())
    }
  }
}

#[derive(Debug, Clone)]
pub struct UxROMPPUMemoryInterceptor {
  chr_rom: [u8; 8 * 1024],
  bus: PPUMemory,
}

impl BusInterceptor<u16> for UxROMPPUMemoryInterceptor {
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
pub struct UxROM {
  cpu_bus_interceptor: UxROMCPUBusInterceptor,
}

impl Mapper for UxROM {
  type CPUBusInterceptor = UxROMCPUBusInterceptor;
  type PPUMemoryInterceptor = UxROMPPUMemoryInterceptor;

  fn from_ines_rom(rom: crate::nes::INESRom) -> Self
  where
    Self: Sized,
  {
    let mut chr_rom: [u8; 8 * 1024] = [0; 8 * 1024];
    if !rom.chr_data.is_empty() {
      for chunk in chr_rom.chunks_exact_mut(rom.chr_data.len()) {
        chunk.copy_from_slice(&rom.chr_data);
      }
    }

    let ppu_memory_interceptor = UxROMPPUMemoryInterceptor {
      bus: PPUMemory::new(rom.initial_mirroring()),
      chr_rom,
    };
    let cpu_bus_interceptor = UxROMCPUBusInterceptor {
      bus: CPUBus::new(PPUCPUBus::new(Box::new(ppu_memory_interceptor))),
      prg_rom: rom.prg_data,
      bank_select: 0,
    };

    Self {
      cpu_bus_interceptor,
    }
  }

  fn cpu_bus(&self) -> &UxROMCPUBusInterceptor {
    &self.cpu_bus_interceptor
  }

  fn cpu_bus_mut(&mut self) -> &mut UxROMCPUBusInterceptor {
    &mut self.cpu_bus_interceptor
  }
}
