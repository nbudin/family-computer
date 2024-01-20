use bitfield_struct::bitfield;

use crate::{
  cpu::CPUBus,
  ppu::{PPUCPUBus, PPUMemory},
};

use super::{
  bus_interceptor::{BusInterceptor, InterceptorResult},
  CartridgeMirroring, Mapper,
};

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

#[derive(Debug, Clone)]
pub struct MMC1CPUBusInterceptor {
  pub prg_rom: Vec<u8>,
  pub control: MMC1ControlRegister,
  pub prg_bank_select: u8,
  pub prg_ram_bank_select: u8,
  pub prg_ram: [u8; 32 * 1024],
  pub shift_register: MMC1ShiftRegister,
  bus: CPUBus<MMC1PPUMemoryInterceptor>,
}

impl BusInterceptor<u16> for MMC1CPUBusInterceptor {
  type BusType = CPUBus<MMC1PPUMemoryInterceptor>;

  fn get_inner(&self) -> &CPUBus<MMC1PPUMemoryInterceptor> {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut CPUBus<MMC1PPUMemoryInterceptor> {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      let offset = (addr - 0x6000) as usize;

      let prg_ram_addr = (0x2000 * (self.prg_ram_bank_select as usize)) + offset;
      InterceptorResult::Intercepted(Some(self.prg_ram[prg_ram_addr % self.prg_ram.len()]))
    } else if addr < 0xc000 {
      let offset = (addr - 0x8000) as usize;

      let prg_addr = match self.control.prg_rom_bank_mode() {
        MMC1PRGROMBankMode::Full32KB1 | MMC1PRGROMBankMode::Full32KB2 => {
          (0x8000 * (self.prg_bank_select as usize)) + offset
        }
        MMC1PRGROMBankMode::FixedLow => offset,
        MMC1PRGROMBankMode::FixedHigh => (0x4000 * (self.prg_bank_select as usize)) + offset,
      };

      InterceptorResult::Intercepted(Some(self.prg_rom[prg_addr % self.prg_rom.len()]))
    } else {
      let offset = (addr - 0xc000) as usize;

      let prg_addr = match self.control.prg_rom_bank_mode() {
        MMC1PRGROMBankMode::Full32KB1 | MMC1PRGROMBankMode::Full32KB2 => {
          (0x8000 * (self.prg_bank_select as usize)) + offset + 0x2000
        }
        MMC1PRGROMBankMode::FixedLow => (0x4000 * (self.prg_bank_select as usize)) + offset,
        MMC1PRGROMBankMode::FixedHigh => (self.prg_rom.len() - 0x4000) + offset,
      };

      InterceptorResult::Intercepted(Some(self.prg_rom[prg_addr % self.prg_rom.len()]))
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      let offset = (addr - 0x6000) as usize;

      let prg_ram_addr = (0x2000 * (self.prg_ram_bank_select as usize)) + offset;
      let prg_ram_size = self.prg_ram.len();
      self.prg_ram[prg_ram_addr % prg_ram_size] = value;
      InterceptorResult::Intercepted(())
    } else {
      if value & (1 << 7) > 0 {
        self.shift_register.reset();
        self
          .control
          .set_prg_rom_bank_mode(MMC1PRGROMBankMode::FixedHigh);
      } else {
        // TODO: ignore consecutive-cycle writes?
        let result = self.shift_register.write_bit(value & 0b1 == 1);

        if let Some(data) = result {
          if addr < 0xa000 {
            self.control = data.into();
            self.bus.ppu_cpu_bus.ppu_memory.control = self.control;
            self.bus.ppu_cpu_bus.ppu_memory.get_inner_mut().mirroring =
              match self.control.mirroring() {
                MMC1MirroringMode::SingleScreenLow | MMC1MirroringMode::SingleScreenHigh => {
                  CartridgeMirroring::SingleScreen
                }
                MMC1MirroringMode::Vertical => CartridgeMirroring::Vertical,
                MMC1MirroringMode::Horizontal => CartridgeMirroring::Horizontal,
              }
          } else if addr < 0xc000 {
            self.bus.ppu_cpu_bus.ppu_memory.chr_low_bank_select = data;
          } else if addr < 0xe000 {
            self.bus.ppu_cpu_bus.ppu_memory.chr_high_bank_select = data;
          } else {
            // TODO deal with high bit
            self.prg_bank_select = data & 0b1111;
          }
        }
      }

      InterceptorResult::Intercepted(())
    }
  }
}

#[derive(Debug, Clone)]
pub struct MMC1PPUMemoryInterceptor {
  bus: PPUMemory,
  pub control: MMC1ControlRegister,
  pub chr_low_bank_select: u8,
  pub chr_high_bank_select: u8,
  pub chr_mem: Vec<u8>,
}

impl BusInterceptor<u16> for MMC1PPUMemoryInterceptor {
  type BusType = PPUMemory;

  fn get_inner(&self) -> &PPUMemory {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut PPUMemory {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    if addr < 0x1000 {
      let offset = addr as usize;

      let chr_addr = match self.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => (self.chr_low_bank_select as usize * 8 * 1024) + offset,
        MMC1CHRROMBankMode::Split4KB => (self.chr_low_bank_select as usize * 4 * 1024) + offset,
      };

      InterceptorResult::Intercepted(Some(self.chr_mem[chr_addr % self.chr_mem.len()]))
    } else if addr < 0x2000 {
      let offset = (addr - 0x1000) as usize;

      let chr_addr = match self.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => {
          // high bank select is ignored in 8kb mode
          (self.chr_low_bank_select as usize * 8 * 1024) + offset + 0x1000
        }
        MMC1CHRROMBankMode::Split4KB => (self.chr_high_bank_select as usize * 4 * 1024) + offset,
      };

      InterceptorResult::Intercepted(Some(self.chr_mem[chr_addr % self.chr_mem.len()]))
    } else {
      InterceptorResult::NotIntercepted
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x1000 {
      let offset = addr as usize;

      let chr_addr = match self.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => (self.chr_low_bank_select as usize * 8 * 1024) + offset,
        MMC1CHRROMBankMode::Split4KB => (self.chr_low_bank_select as usize * 4 * 1024) + offset,
      };

      let chr_mem_size = self.chr_mem.len();
      self.chr_mem[chr_addr % chr_mem_size] = value;

      InterceptorResult::Intercepted(())
    } else if addr < 0x2000 {
      let offset = (addr - 0x1000) as usize;

      let chr_addr = match self.control.chr_rom_bank_mode() {
        MMC1CHRROMBankMode::Full8KB => {
          // high bank select is ignored in 8kb mode
          (self.chr_low_bank_select as usize * 8 * 1024) + offset + 0x1000
        }
        MMC1CHRROMBankMode::Split4KB => (self.chr_high_bank_select as usize * 4 * 1024) + offset,
      };

      let chr_mem_size = self.chr_mem.len();
      self.chr_mem[chr_addr % chr_mem_size] = value;

      InterceptorResult::Intercepted(())
    } else {
      InterceptorResult::NotIntercepted
    }
  }
}

#[derive(Debug, Clone)]
pub struct MMC1 {
  cpu_bus: MMC1CPUBusInterceptor,
}

impl Mapper for MMC1 {
  type CPUBusInterceptor = MMC1CPUBusInterceptor;
  type PPUMemoryInterceptor = MMC1PPUMemoryInterceptor;

  fn from_ines_rom(rom: crate::nes::INESRom) -> Self
  where
    Self: Sized,
  {
    let chr_data = if rom.uses_chr_ram {
      Vec::from([0; 1024 * 128])
    } else {
      rom.chr_data.clone()
    };

    let ppu_memory_interceptor = MMC1PPUMemoryInterceptor {
      bus: PPUMemory::new(rom.initial_mirroring()),
      chr_high_bank_select: 0,
      chr_low_bank_select: 0,
      chr_mem: chr_data,
      control: MMC1ControlRegister(0).with_prg_rom_bank_mode(MMC1PRGROMBankMode::FixedHigh),
    };

    let cpu_bus = MMC1CPUBusInterceptor {
      bus: CPUBus::new(PPUCPUBus::new(Box::new(ppu_memory_interceptor))),
      control: MMC1ControlRegister(0).with_prg_rom_bank_mode(MMC1PRGROMBankMode::FixedHigh),
      prg_rom: rom.prg_data,
      prg_bank_select: 0,
      prg_ram_bank_select: 0,
      prg_ram: [0; 32 * 1024],
      shift_register: MMC1ShiftRegister::new(),
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
