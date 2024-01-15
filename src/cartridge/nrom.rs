use std::sync::{Arc, RwLock};

use crate::{
  bus::{BusInterceptor, InterceptorResult},
  cpu::CPUBus,
  nes::INESRom,
  ppu::PPUMemory,
};

use super::{Cartridge, CartridgeMirroring, CartridgeState};

#[derive(Debug, Clone)]
pub struct NROMState {
  pub chr_rom: [u8; 8 * 1024],
  pub prg_ram: [u8; 8 * 1024],
}

impl NROMState {
  fn new(chr_rom: [u8; 8 * 1024]) -> Self {
    Self {
      chr_rom,
      prg_ram: [0; 8 * 1024],
    }
  }
}

impl CartridgeState for NROMState {}

struct NROMCPUBusInterceptor<'a> {
  cartridge: &'a NROM,
  bus: CPUBus<'a>,
}

impl<'a> BusInterceptor<'a, u16> for NROMCPUBusInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      let state = self.cartridge.state.read().unwrap();
      InterceptorResult::Intercepted(Some(state.prg_ram[usize::from(addr) % (8 * 1024)]))
    } else {
      InterceptorResult::Intercepted(Some(self.cartridge.prg_rom[usize::from(addr - 0x8000)]))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      let mut state = self.cartridge.state.write().unwrap();
      state.prg_ram[usize::from(addr) % (8 * 1024)] = value;
      InterceptorResult::Intercepted(())
    } else {
      // can't write to rom
      InterceptorResult::Intercepted(())
    }
  }
}

struct NROMPPUMemoryInterceptor<'a> {
  cartridge: &'a NROM,
  bus: PPUMemory<'a>,
}

impl<'a> BusInterceptor<'a, u16> for NROMPPUMemoryInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x2000 {
      let state = self.cartridge.state.read().unwrap();
      InterceptorResult::Intercepted(Some(state.chr_rom[usize::from(addr)]))
    } else {
      InterceptorResult::NotIntercepted
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x2000 {
      let mut state = self.cartridge.state.write().unwrap();
      state.chr_rom[usize::from(addr)] = value;
      InterceptorResult::Intercepted(())
    } else {
      InterceptorResult::NotIntercepted
    }
  }
}

#[derive(Debug, Clone)]
pub struct NROM {
  pub prg_rom: [u8; 32 * 1024],
  pub state: Arc<RwLock<NROMState>>,
  mirroring: CartridgeMirroring,
}

impl Cartridge for NROM {
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

    Self {
      prg_rom,
      state: Arc::new(RwLock::new(NROMState::new(chr_rom))),
      mirroring: if rom.vertical_mirroring {
        CartridgeMirroring::Vertical
      } else {
        CartridgeMirroring::Horizontal
      },
    }
  }

  fn cpu_bus_interceptor<'a>(&'a self, bus: CPUBus<'a>) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(NROMCPUBusInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn cpu_bus_interceptor_mut<'a>(
    &'a self,
    bus: CPUBus<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(NROMCPUBusInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn ppu_memory_interceptor<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(NROMPPUMemoryInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn ppu_memory_interceptor_mut<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(NROMPPUMemoryInterceptor {
      cartridge: self,
      bus,
    })
  }

  fn get_mirroring(&self) -> CartridgeMirroring {
    self.mirroring
  }
}
