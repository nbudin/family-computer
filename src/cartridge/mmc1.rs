use bitfield_struct::bitfield;

use crate::{
  bus::{BusInterceptor, InterceptorResult, RwHandle},
  cpu::CPUBus,
  ppu::PPUMemory,
};

use super::{Cartridge, CartridgeMirroring, CartridgeState};

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MMC1MirroringMode {
  SingleScreenLow = 0,
  SingleScreenHigh = 1,
  Vertical = 2,
  Horizontal = 3,
}

impl MMC1MirroringMode {
  const fn into_bits(self) -> u8 {
    self as _
  }

  const fn from_bits(value: u8) -> Self {
    match value {
      0 => Self::SingleScreenLow,
      1 => Self::SingleScreenHigh,
      2 => Self::Vertical,
      _ => Self::Horizontal,
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MMC1PRGROMBankMode {
  Full32KB1 = 0,
  Full32KB2 = 1,
  FixedLow = 2,
  FixedHigh = 3,
}

impl MMC1PRGROMBankMode {
  const fn into_bits(self) -> u8 {
    self as _
  }

  const fn from_bits(value: u8) -> Self {
    match value {
      0 => Self::Full32KB1,
      1 => Self::Full32KB2,
      2 => Self::FixedLow,
      _ => Self::FixedHigh,
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MMC1CHRROMBankMode {
  Full8KB = 0,
  Split4KB = 1,
}

impl MMC1CHRROMBankMode {
  const fn into_bits(self) -> u8 {
    self as _
  }

  const fn from_bits(value: u8) -> Self {
    match value {
      0 => Self::Full8KB,
      _ => Self::Split4KB,
    }
  }
}

#[bitfield(u8)]
pub struct MMC1ControlRegister {
  #[bits(2)]
  pub mirroring: MMC1MirroringMode,
  #[bits(2)]
  pub prg_rom_bank_mode: MMC1PRGROMBankMode,
  #[bits(1)]
  pub chr_rom_bank_mode: MMC1CHRROMBankMode,
  #[bits(3)]
  _unused: u8,
}

#[derive(Debug, Clone)]
pub struct MMC1ShiftRegister {
  pub value: u8,
}

impl MMC1ShiftRegister {
  pub fn new() -> Self {
    Self { value: 0b10000 }
  }

  pub fn write_bit(&mut self, value: bool) -> Option<u8> {
    let last_bit = self.value & 0b1 == 1;
    let new_value = (self.value >> 1) | ((value as u8) << 4);

    if last_bit {
      self.reset();
      Some(new_value)
    } else {
      self.value = new_value;
      None
    }
  }

  pub fn reset(&mut self) {
    self.value = 0b10000;
  }
}

#[derive(Debug, Clone)]
pub struct MMC1State {
  pub chr_mem: Vec<u8>,
  pub control: MMC1ControlRegister,
  pub prg_bank_select: u8,
  pub chr_low_bank_select: u8,
  pub chr_high_bank_select: u8,
  pub prg_ram_bank_select: u8,
  pub prg_ram: [u8; 32 * 1024],
  pub shift_register: MMC1ShiftRegister,
}

impl MMC1State {
  fn new(chr_data: Vec<u8>) -> Self {
    Self {
      chr_mem: chr_data,
      control: MMC1ControlRegister(0).with_prg_rom_bank_mode(MMC1PRGROMBankMode::FixedHigh),
      prg_bank_select: 0,
      chr_low_bank_select: 0,
      chr_high_bank_select: 0,
      prg_ram_bank_select: 0,
      prg_ram: [0; 32 * 1024],
      shift_register: MMC1ShiftRegister::new(),
    }
  }
}

impl CartridgeState for MMC1State {}

struct MMC1CPUBusInterceptor<'a> {
  cartridge: RwHandle<'a, MMC1>,
  bus: CPUBus<'a>,
}

impl<'a> BusInterceptor<'a, u16> for MMC1CPUBusInterceptor<'a> {
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
      let offset = (addr - 0x6000) as usize;
      let prg_ram_addr = (0x2000 * (self.cartridge.state.prg_ram_bank_select as usize)) + offset;
      InterceptorResult::Intercepted(Some(
        self.cartridge.state.prg_ram[prg_ram_addr % self.cartridge.state.prg_ram.len()],
      ))
    } else if addr < 0xc000 {
      let offset = (addr - 0x8000) as usize;

      let prg_addr = match self.cartridge.state.control.prg_rom_bank_mode() {
        MMC1PRGROMBankMode::Full32KB1 | MMC1PRGROMBankMode::Full32KB2 => {
          (0x8000 * (self.cartridge.state.prg_bank_select as usize)) + offset
        }
        MMC1PRGROMBankMode::FixedLow => offset,
        MMC1PRGROMBankMode::FixedHigh => {
          (0x4000 * (self.cartridge.state.prg_bank_select as usize)) + offset
        }
      };

      InterceptorResult::Intercepted(Some(
        self.cartridge.prg_rom[prg_addr % self.cartridge.prg_rom.len()],
      ))
    } else {
      let offset = (addr - 0xc000) as usize;

      let prg_addr = match self.cartridge.state.control.prg_rom_bank_mode() {
        MMC1PRGROMBankMode::Full32KB1 | MMC1PRGROMBankMode::Full32KB2 => {
          (0x8000 * (self.cartridge.state.prg_bank_select as usize)) + offset + 0x2000
        }
        MMC1PRGROMBankMode::FixedLow => {
          (0x4000 * (self.cartridge.state.prg_bank_select as usize)) + offset
        }
        MMC1PRGROMBankMode::FixedHigh => (self.cartridge.prg_rom.len() - 0x4000) + offset,
      };

      InterceptorResult::Intercepted(Some(
        self.cartridge.prg_rom[prg_addr % self.cartridge.prg_rom.len()],
      ))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      let offset = (addr - 0x6000) as usize;
      let prg_ram_addr = (0x2000 * (self.cartridge.state.prg_ram_bank_select as usize)) + offset;
      let prg_ram_size = self.cartridge.state.prg_ram.len();
      self.cartridge.get_mut().state.prg_ram[prg_ram_addr % prg_ram_size] = value;
      InterceptorResult::Intercepted(())
    } else {
      if value & (1 << 7) > 0 {
        let cartridge = self.cartridge.get_mut();
        cartridge.state.shift_register.reset();
        cartridge
          .state
          .control
          .set_prg_rom_bank_mode(MMC1PRGROMBankMode::FixedHigh);
      } else {
        // TODO: ignore consecutive-cycle writes?
        let result = self
          .cartridge
          .get_mut()
          .state
          .shift_register
          .write_bit(value & 0b1 == 1);

        if let Some(data) = result {
          if addr < 0xa000 {
            self.cartridge.get_mut().state.control = data.into();
          } else if addr < 0xc000 {
            self.cartridge.get_mut().state.chr_low_bank_select = data;
          } else if addr < 0xe000 {
            self.cartridge.get_mut().state.chr_high_bank_select = data;
          } else {
            // TODO deal with high bit
            self.cartridge.get_mut().state.prg_bank_select = data & 0b1111;
          }
        }
      }

      InterceptorResult::Intercepted(())
    }
  }
}

struct MMC1PPUMemoryInterceptor<'a> {
  bus: PPUMemory<'a>,
  cartridge: RwHandle<'a, MMC1>,
}

impl<'a> BusInterceptor<'a, u16> for MMC1PPUMemoryInterceptor<'a> {
  fn bus(&self) -> &dyn crate::bus::Bus<u16> {
    &self.bus
  }

  fn bus_mut(&mut self) -> &mut dyn crate::bus::Bus<u16> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x1000 {
      let offset = addr as usize;

      let chr_addr = match self.cartridge.state.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => {
          (self.cartridge.state.chr_low_bank_select as usize * 8 * 1024) + offset
        }
        MMC1CHRROMBankMode::Split4KB => {
          (self.cartridge.state.chr_low_bank_select as usize * 4 * 1024) + offset
        }
      };

      InterceptorResult::Intercepted(Some(
        self.cartridge.state.chr_mem[chr_addr % self.cartridge.state.chr_mem.len()],
      ))
    } else if addr < 0x2000 {
      let offset = (addr - 0x1000) as usize;

      let chr_addr = match self.cartridge.state.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => {
          // high bank select is ignored in 8kb mode
          (self.cartridge.state.chr_low_bank_select as usize * 8 * 1024) + offset + 0x1000
        }
        MMC1CHRROMBankMode::Split4KB => {
          (self.cartridge.state.chr_high_bank_select as usize * 4 * 1024) + offset
        }
      };

      InterceptorResult::Intercepted(Some(
        self.cartridge.state.chr_mem[chr_addr % self.cartridge.state.chr_mem.len()],
      ))
    } else {
      InterceptorResult::NotIntercepted
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x1000 {
      let offset = addr as usize;

      let chr_addr = match self.cartridge.state.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => {
          (self.cartridge.state.chr_low_bank_select as usize * 8 * 1024) + offset
        }
        MMC1CHRROMBankMode::Split4KB => {
          (self.cartridge.state.chr_low_bank_select as usize * 4 * 1024) + offset
        }
      };

      let chr_mem_size = self.cartridge.state.chr_mem.len();
      self.cartridge.get_mut().state.chr_mem[chr_addr % chr_mem_size] = value;

      InterceptorResult::Intercepted(())
    } else if addr < 0x2000 {
      let offset = (addr - 0x1000) as usize;

      let chr_addr = match self.cartridge.state.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => {
          // high bank select is ignored in 8kb mode
          (self.cartridge.state.chr_low_bank_select as usize * 8 * 1024) + offset + 0x1000
        }
        MMC1CHRROMBankMode::Split4KB => {
          (self.cartridge.state.chr_high_bank_select as usize * 4 * 1024) + offset
        }
      };

      let chr_mem_size = self.cartridge.state.chr_mem.len();
      self.cartridge.get_mut().state.chr_mem[chr_addr % chr_mem_size] = value;

      InterceptorResult::Intercepted(())
    } else {
      InterceptorResult::NotIntercepted
    }
  }
}

#[derive(Debug, Clone)]
pub struct MMC1 {
  pub prg_rom: Vec<u8>,
  pub state: MMC1State,
}

impl Cartridge for MMC1 {
  fn from_ines_rom(rom: crate::nes::INESRom) -> Self
  where
    Self: Sized,
  {
    Self {
      prg_rom: rom.prg_data,
      state: MMC1State::new(if rom.uses_chr_ram {
        Vec::from([0; 1024 * 128])
      } else {
        rom.chr_data
      }),
    }
  }

  fn cpu_bus_interceptor<'a>(&'a self, bus: CPUBus<'a>) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(MMC1CPUBusInterceptor {
      bus,
      cartridge: RwHandle::ReadOnly(self),
    })
  }

  fn cpu_bus_interceptor_mut<'a>(
    &'a mut self,
    bus: CPUBus<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(MMC1CPUBusInterceptor {
      bus,
      cartridge: RwHandle::ReadWrite(self),
    })
  }

  fn ppu_memory_interceptor<'a>(
    &'a self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(MMC1PPUMemoryInterceptor {
      bus,
      cartridge: RwHandle::ReadOnly(self),
    })
  }

  fn ppu_memory_interceptor_mut<'a>(
    &'a mut self,
    bus: PPUMemory<'a>,
  ) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    Box::new(MMC1PPUMemoryInterceptor {
      bus,
      cartridge: RwHandle::ReadWrite(self),
    })
  }

  fn get_mirroring(&self) -> CartridgeMirroring {
    match self.state.control.mirroring() {
      MMC1MirroringMode::SingleScreenLow | MMC1MirroringMode::SingleScreenHigh => {
        CartridgeMirroring::SingleScreen
      }
      MMC1MirroringMode::Vertical => CartridgeMirroring::Vertical,
      MMC1MirroringMode::Horizontal => CartridgeMirroring::Horizontal,
    }
  }
}
