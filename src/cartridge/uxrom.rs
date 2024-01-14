use crate::{
  bus::{BusInterceptor, InterceptorResult, RwHandle},
  cpu::CPUBus,
  ppu::PPUMemory,
};

use super::{Cartridge, CartridgeMirroring, CartridgeState};

#[derive(Debug, Clone)]
pub struct UxROMState {
  pub bank_select: u8,
}

impl UxROMState {
  fn new() -> Self {
    Self { bank_select: 0 }
  }
}

impl CartridgeState for UxROMState {}

struct UxROMCPUBusInterceptor<'a> {
  cartridge: RwHandle<'a, UxROM>,
  bus: CPUBus<'a>,
}

impl<'a> BusInterceptor<'a, u16> for UxROMCPUBusInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    // TODO bank switching
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0xc000 {
      InterceptorResult::Intercepted(Some(
        *self
          .cartridge
          .prg_rom
          .get(
            ((0x4000 * (self.cartridge.state.bank_select as usize)) + usize::from(addr - 0x8000))
              % self.cartridge.prg_rom.len(),
          )
          .unwrap(),
      ))
    } else {
      InterceptorResult::Intercepted(Some(
        self.cartridge.prg_rom[self.cartridge.prg_rom.len() - 0x4000 + usize::from(addr - 0xc000)],
      ))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x8000 {
      InterceptorResult::NotIntercepted
    } else {
      self.cartridge.get_mut().state.bank_select = value;
      InterceptorResult::Intercepted(())
    }
  }
}

struct UxROMPPUMemoryInterceptor<'a> {
  bus: PPUMemory<'a>,
}

impl<'a> BusInterceptor<'a, u16> for UxROMPPUMemoryInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, _addr: u16) -> InterceptorResult<Option<u8>> {
    InterceptorResult::NotIntercepted
  }

  fn intercept_write(&mut self, _addr: u16, _value: u8) -> InterceptorResult<()> {
    InterceptorResult::NotIntercepted
  }
}

#[derive(Debug, Clone)]
pub struct UxROM {
  pub prg_rom: Vec<u8>,
  pub chr_rom: [u8; 8 * 1024],
  pub state: UxROMState,
  mirroring: CartridgeMirroring,
}

impl Cartridge for UxROM {
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

    Self {
      prg_rom: rom.prg_data,
      chr_rom,
      state: UxROMState::new(),
      mirroring: if rom.vertical_mirroring {
        CartridgeMirroring::VERTICAL
      } else {
        CartridgeMirroring::HORIZONTAL
      },
    }
  }

  fn cpu_bus_interceptor<'a>(&'a self, bus: CPUBus<'a>) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(UxROMCPUBusInterceptor {
      bus,
      cartridge: RwHandle::ReadOnly(self),
    })
  }

  fn cpu_bus_interceptor_mut<'a>(
    &'a mut self,
    bus: CPUBus<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(UxROMCPUBusInterceptor {
      bus,
      cartridge: RwHandle::ReadWrite(self),
    })
  }

  fn ppu_memory_interceptor<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(UxROMPPUMemoryInterceptor { bus })
  }

  fn ppu_memory_interceptor_mut<'a>(
    &'a mut self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(UxROMPPUMemoryInterceptor { bus })
  }

  fn get_mirroring(&self) -> CartridgeMirroring {
    self.mirroring
  }
}
