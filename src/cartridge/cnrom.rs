use std::sync::{Arc, RwLock};

use crate::{
  bus::{BusInterceptor, InterceptorResult},
  cpu::CPUBus,
  nes::INESRom,
  ppu::PPUMemory,
};

use super::{Cartridge, CartridgeMirroring, CartridgeState};

#[derive(Debug, Clone)]
pub struct CNROMState {
  pub bank_select: u8,
  pub chr_rom: [u8; 4 * 8 * 1024],
}

impl CNROMState {
  fn new(chr_rom: [u8; 4 * 8 * 1024]) -> Self {
    Self {
      bank_select: 0,
      chr_rom,
    }
  }
}

impl CartridgeState for CNROMState {}

struct CNROMCPUBusInterceptor<'a> {
  cartridge: &'a CNROM,
  bus: CPUBus<'a>,
}

impl<'a> BusInterceptor<'a, u16> for CNROMCPUBusInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else {
      InterceptorResult::Intercepted(Some(self.cartridge.prg_rom[usize::from(addr - 0x8000)]))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else {
      let mut state = self.cartridge.state.write().unwrap();
      state.bank_select = value & 0b11;
      InterceptorResult::Intercepted(())
    }
  }
}

struct CNROMPPUMemoryInterceptor<'a> {
  cartridge: &'a CNROM,
  bus: PPUMemory<'a>,
}

impl<'a> BusInterceptor<'a, u16> for CNROMPPUMemoryInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x2000 {
      let state = self.cartridge.state.read().unwrap();
      InterceptorResult::Intercepted(Some(
        state.chr_rom[(state.bank_select as usize * 8 * 1024) + usize::from(addr)],
      ))
    } else {
      InterceptorResult::NotIntercepted
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x2000 {
      let mut state = self.cartridge.state.write().unwrap();
      let addr = (state.bank_select as usize * 8 * 1024) + usize::from(addr);
      state.chr_rom[addr] = value;
      InterceptorResult::Intercepted(())
    } else {
      InterceptorResult::NotIntercepted
    }
  }
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct CNROM {
  pub prg_rom: [u8; 32 * 1024],
  pub state: Arc<RwLock<CNROMState>>,
  mirroring: CartridgeMirroring,
}

impl Cartridge for CNROM {
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

    Self {
      prg_rom,
      state: Arc::new(RwLock::new(CNROMState::new(chr_rom))),
      mirroring: if rom.vertical_mirroring {
        CartridgeMirroring::Vertical
      } else {
        CartridgeMirroring::Horizontal
      },
    }
  }

  fn cpu_bus_interceptor<'a>(&'a self, bus: CPUBus<'a>) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(CNROMCPUBusInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn cpu_bus_interceptor_mut<'a>(
    &'a self,
    bus: CPUBus<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(CNROMCPUBusInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn ppu_memory_interceptor<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(CNROMPPUMemoryInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn ppu_memory_interceptor_mut<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(CNROMPPUMemoryInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn get_mirroring(&self) -> CartridgeMirroring {
    self.mirroring
  }
}
