use bitfield_struct::bitfield;

use crate::{
  cpu::CPUBus,
  ppu::{PPUCPUBus, PPUMemory},
};

use super::{
  bus_interceptor::{BusInterceptor, InterceptorResult},
  CartridgeMirroring, Mapper,
};

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum MMC3TargetBank {
  R0 = 0b000,
  R1 = 0b001,
  R2 = 0b010,
  R3 = 0b011,
  R4 = 0b100,
  R5 = 0b101,
  R6 = 0b110,
  R7 = 0b111,
}

impl MMC3TargetBank {
  const fn into_bits(self) -> u8 {
    self as _
  }

  const fn from_bits(value: u8) -> Self {
    match value {
      0b000 => Self::R0,
      0b001 => Self::R1,
      0b010 => Self::R2,
      0b011 => Self::R3,
      0b100 => Self::R4,
      0b101 => Self::R5,
      0b110 => Self::R6,
      _ => Self::R7,
    }
  }
}

#[derive(Debug, Clone)]
pub enum MMC3BankSpecifier {
  Mapped(u8, usize),
  Relative(i8, usize),
}

impl MMC3BankSpecifier {
  pub fn resolve_bank_index(&self, mapping: &[u8], data: &Vec<u8>) -> usize {
    match self {
      MMC3BankSpecifier::Mapped(index, _) => mapping[*index as usize] as usize,
      MMC3BankSpecifier::Relative(bank_offset, bank_size) => {
        if *bank_offset > 0 {
          *bank_offset as usize
        } else {
          let bank_count = data.len() / bank_size;
          (bank_count as isize + *bank_offset as isize) as usize
        }
      }
    }
  }

  pub fn resolve_addr(&self, mapping: &[u8], data: &Vec<u8>, offset: u16) -> usize {
    match self {
      MMC3BankSpecifier::Mapped(_, bank_size) | MMC3BankSpecifier::Relative(_, bank_size) => {
        let bank_index = self.resolve_bank_index(mapping, data);
        (bank_index * bank_size) + offset as usize
      }
    }
  }
}

// R0 and R1 values are always multiples of 2, so we should treat them as pointing to 1KB banks
const CHR_R0: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(0, 1024);
const CHR_R1: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(1, 1024);
const CHR_R2: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(2, 1024);
const CHR_R3: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(3, 1024);
const CHR_R4: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(4, 1024);
const CHR_R5: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(5, 1024);
const PRG_R6: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(0, 8 * 1024);
const PRG_R7: MMC3BankSpecifier = MMC3BankSpecifier::Mapped(1, 8 * 1024);
const PRG_LAST: MMC3BankSpecifier = MMC3BankSpecifier::Relative(-1, 8 * 1024);
const PRG_SECOND_LAST: MMC3BankSpecifier = MMC3BankSpecifier::Relative(-2, 8 * 1024);

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum MMC3PRGROMBankMode {
  FixedHigh = 0,
  FixedLow = 1,
}

impl MMC3PRGROMBankMode {
  const fn into_bits(self) -> u8 {
    self as _
  }

  const fn from_bits(value: u8) -> Self {
    match value {
      0 => Self::FixedHigh,
      _ => Self::FixedLow,
    }
  }
}

#[bitfield(u8)]
pub struct MMC3BankSelectRegister {
  #[bits(3)]
  target_bank: MMC3TargetBank,
  #[bits(3)]
  _unused: u8,
  #[bits(1)]
  prg_rom_bank_mode: MMC3PRGROMBankMode,
  chr_a12_inversion: bool,
}

#[derive(Debug, Clone)]
pub struct MMC3CPUBusInterceptor {
  pub prg_rom: Vec<u8>,
  pub prg_ram: [u8; 8 * 1024],
  pub four_screen_vram: bool,
  pub prg_rom_bank_mode: MMC3PRGROMBankMode,
  pub selected_bank: MMC3TargetBank,
  pub prg_rom_bank_mapping: [u8; 2],
  pub irq_enabled: bool,
  pub irq_reload: u8,
  pub irq_counter: u8,
  pub pending_irq: bool,
  pub irq_reload_pending: bool,
  bus: CPUBus<MMC3PPUMemoryInterceptor>,
}

impl MMC3CPUBusInterceptor {
  fn read_banked(&self, bank: MMC3BankSpecifier, offset: u16) -> u8 {
    let addr = bank.resolve_addr(&self.prg_rom_bank_mapping, &self.prg_rom, offset);
    self.prg_rom[addr]
  }
}

impl BusInterceptor<u16> for MMC3CPUBusInterceptor {
  type BusType = CPUBus<MMC3PPUMemoryInterceptor>;

  fn get_inner(&self) -> &Self::BusType {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut Self::BusType {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    match addr {
      0x0000..=0x5fff => InterceptorResult::NotIntercepted,
      0x6000..=0x7fff => InterceptorResult::Intercepted(Some(self.prg_ram[addr as usize - 0x6000])),
      0x8000..=0x9fff => InterceptorResult::Intercepted(Some(match self.prg_rom_bank_mode {
        MMC3PRGROMBankMode::FixedHigh => self.read_banked(PRG_R6, addr - 0x8000),
        MMC3PRGROMBankMode::FixedLow => self.read_banked(PRG_SECOND_LAST, addr - 0x8000),
      })),
      0xa000..=0xbfff => {
        InterceptorResult::Intercepted(Some(self.read_banked(PRG_R7, addr - 0xa000)))
      }
      0xc000..=0xdfff => InterceptorResult::Intercepted(Some(match self.prg_rom_bank_mode {
        MMC3PRGROMBankMode::FixedHigh => self.read_banked(PRG_SECOND_LAST, addr - 0xc000),
        MMC3PRGROMBankMode::FixedLow => self.read_banked(PRG_R6, addr - 0xc000),
      })),
      0xe000..=0xffff => {
        InterceptorResult::Intercepted(Some(self.read_banked(PRG_LAST, addr - 0xe000)))
      }
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    if addr < 0x6000 {
      InterceptorResult::NotIntercepted
    } else if addr < 0x8000 {
      self.prg_ram[addr as usize - 0x6000] = value;
      InterceptorResult::Intercepted(())
    } else if addr < 0xa000 {
      if addr % 2 == 0 {
        // even addresses write the bank select register
        let value = MMC3BankSelectRegister::from(value);
        self.selected_bank = value.target_bank();
        self.prg_rom_bank_mode = value.prg_rom_bank_mode();
        self.bus.ppu_cpu_bus.ppu_memory.chr_a12_inversion = value.chr_a12_inversion();
      } else {
        // odd addresses set a bank mapping
        match self.selected_bank {
          MMC3TargetBank::R0 => {
            self.bus.ppu_cpu_bus.ppu_memory.chr_bank_mapping[0] = value & 0b11111110
          }
          MMC3TargetBank::R1 => {
            self.bus.ppu_cpu_bus.ppu_memory.chr_bank_mapping[1] = value & 0b11111110
          }
          MMC3TargetBank::R2 => self.bus.ppu_cpu_bus.ppu_memory.chr_bank_mapping[2] = value,
          MMC3TargetBank::R3 => self.bus.ppu_cpu_bus.ppu_memory.chr_bank_mapping[3] = value,
          MMC3TargetBank::R4 => self.bus.ppu_cpu_bus.ppu_memory.chr_bank_mapping[4] = value,
          MMC3TargetBank::R5 => self.bus.ppu_cpu_bus.ppu_memory.chr_bank_mapping[5] = value,
          MMC3TargetBank::R6 => self.prg_rom_bank_mapping[0] = value,
          MMC3TargetBank::R7 => self.prg_rom_bank_mapping[1] = value,
        }
      }

      InterceptorResult::Intercepted(())
    } else if addr < 0xc000 {
      if addr % 2 == 0 {
        // even addresses set the mirroring
        if !self.four_screen_vram {
          self.bus.ppu_cpu_bus.ppu_memory.bus.mirroring = match value & 0b1 {
            0 => CartridgeMirroring::Vertical,
            _ => CartridgeMirroring::Horizontal,
          };
        }
      } else {
        // odd addresses set PRG RAM protection
        // which we're deliberately leaving out on MMC3
      }
      InterceptorResult::Intercepted(())
    } else if addr < 0xe000 {
      if addr % 2 == 0 {
        // even addresses set the IRQ counter reload value
        self.irq_reload = value;
      } else {
        // odd addresses clear the IRQ counter
        self.irq_reload_pending = true;
        self.irq_counter = 0;
      }
      InterceptorResult::Intercepted(())
    } else {
      if addr % 2 == 0 {
        // even addresses disables IRQ and acknowledges any pending interrupts
        self.irq_enabled = false;
        self.pending_irq = false;
      } else {
        // odd addresses enable IRQ
        self.irq_enabled = true;
      }
      InterceptorResult::Intercepted(())
    }
  }
}

#[derive(Debug, Clone)]
pub struct MMC3PPUMemoryInterceptor {
  pub chr_data: Vec<u8>,
  pub chr_a12_inversion: bool,
  pub chr_bank_mapping: [u8; 6],
  bus: PPUMemory,
}

impl MMC3PPUMemoryInterceptor {
  fn read_banked(&self, bank: MMC3BankSpecifier, offset: u16) -> u8 {
    let addr = bank.resolve_addr(&self.chr_bank_mapping, &self.chr_data, offset);
    self.chr_data[addr]
  }

  fn write_banked(&mut self, bank: MMC3BankSpecifier, offset: u16, value: u8) {
    let addr = bank.resolve_addr(&self.chr_bank_mapping, &self.chr_data, offset);
    self.chr_data[addr] = value;
  }
}

impl BusInterceptor<u16> for MMC3PPUMemoryInterceptor {
  type BusType = PPUMemory;

  fn get_inner(&self) -> &Self::BusType {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut Self::BusType {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, addr: u16) -> InterceptorResult<Option<u8>> {
    match addr {
      0x0000..=0x1fff => InterceptorResult::Intercepted(Some(match self.chr_a12_inversion {
        false => match addr {
          0x0000..=0x07ff => self.read_banked(CHR_R0, addr),
          0x0800..=0x0fff => self.read_banked(CHR_R1, addr - 0x0800),
          0x1000..=0x13ff => self.read_banked(CHR_R2, addr - 0x1000),
          0x1400..=0x17ff => self.read_banked(CHR_R3, addr - 0x1400),
          0x1800..=0x1bff => self.read_banked(CHR_R4, addr - 0x1800),
          0x1c00..=0xffff => self.read_banked(CHR_R5, addr - 0x1c00),
        },
        true => match addr {
          0x0000..=0x03ff => self.read_banked(CHR_R2, addr),
          0x0400..=0x07ff => self.read_banked(CHR_R3, addr - 0x0400),
          0x0800..=0x0bff => self.read_banked(CHR_R4, addr - 0x0800),
          0x0c00..=0x0fff => self.read_banked(CHR_R5, addr - 0x0c00),
          0x1000..=0x17ff => self.read_banked(CHR_R0, addr - 0x1000),
          0x1800..=0xffff => self.read_banked(CHR_R1, addr - 0x1800),
        },
      })),
      _ => InterceptorResult::NotIntercepted,
    }
  }

  fn intercept_write(&mut self, addr: u16, value: u8) -> InterceptorResult<()> {
    match addr {
      0x0000..=0x1fff => {
        match self.chr_a12_inversion {
          false => match addr {
            0x0000..=0x07ff => self.write_banked(CHR_R0, addr, value),
            0x0800..=0x0fff => self.write_banked(CHR_R1, addr - 0x0800, value),
            0x1000..=0x13ff => self.write_banked(CHR_R2, addr - 0x1000, value),
            0x1400..=0x17ff => self.write_banked(CHR_R3, addr - 0x1400, value),
            0x1800..=0x1bff => self.write_banked(CHR_R4, addr - 0x1800, value),
            0x1c00..=0xffff => self.write_banked(CHR_R5, addr - 0x1c00, value),
          },
          true => match addr {
            0x0000..=0x03ff => self.write_banked(CHR_R2, addr, value),
            0x0400..=0x07ff => self.write_banked(CHR_R3, addr - 0x0400, value),
            0x0800..=0x0bff => self.write_banked(CHR_R4, addr - 0x0800, value),
            0x0c00..=0x0fff => self.write_banked(CHR_R5, addr - 0x0c00, value),
            0x1000..=0x17ff => self.write_banked(CHR_R0, addr - 0x1000, value),
            0x1800..=0xffff => self.write_banked(CHR_R1, addr - 0x1800, value),
          },
        };
        InterceptorResult::Intercepted(())
      }

      _ => InterceptorResult::NotIntercepted,
    }
  }
}

#[derive(Debug, Clone)]
pub struct MMC3 {
  cpu_bus: MMC3CPUBusInterceptor,
}

impl Mapper for MMC3 {
  type CPUBusInterceptor = MMC3CPUBusInterceptor;
  type PPUMemoryInterceptor = MMC3PPUMemoryInterceptor;

  fn from_ines_rom(rom: crate::nes::INESRom) -> Self
  where
    Self: Sized,
  {
    let initial_mirroring = rom.initial_mirroring();
    let chr_data = if rom.uses_chr_ram {
      Vec::from([0; 256 * 1024])
    } else {
      rom.chr_data.clone()
    };

    Self {
      cpu_bus: MMC3CPUBusInterceptor {
        prg_rom: rom.prg_data,
        prg_ram: [0; 8 * 1024],
        four_screen_vram: rom.four_screen_vram,
        prg_rom_bank_mode: MMC3PRGROMBankMode::FixedHigh,
        selected_bank: MMC3TargetBank::R0,
        prg_rom_bank_mapping: [0; 2],
        irq_enabled: false,
        irq_reload: 0,
        irq_counter: 0,
        pending_irq: false,
        irq_reload_pending: false,
        bus: CPUBus::new(PPUCPUBus::new(Box::new(MMC3PPUMemoryInterceptor {
          chr_data,
          bus: PPUMemory::new(initial_mirroring),
          chr_a12_inversion: false,
          chr_bank_mapping: [0; 6],
        }))),
      },
    }
  }

  fn cpu_bus(&self) -> &Self::CPUBusInterceptor {
    &self.cpu_bus
  }

  fn cpu_bus_mut(&mut self) -> &mut Self::CPUBusInterceptor {
    &mut self.cpu_bus
  }

  fn tick_ppu(&mut self, ppu: &mut crate::ppu::PPU, pixbuf: &mut crate::ppu::Pixbuf) -> bool {
    let nmi_set = ppu.tick(
      pixbuf,
      self.cpu_bus_mut().get_inner_mut().ppu_cpu_bus.as_mut(),
    );

    if ppu.cycle == 0 {
      if self.cpu_bus.irq_counter == 0 || self.cpu_bus.irq_reload_pending {
        self.cpu_bus.irq_counter = self.cpu_bus.irq_reload;
        self.cpu_bus.irq_reload_pending = false;
      } else {
        self.cpu_bus.irq_counter -= 1;
      }

      if self.cpu_bus.irq_enabled && self.cpu_bus.irq_counter == 0 {
        self.cpu_bus.pending_irq = true;
      }
    }

    nmi_set
  }

  fn poll_irq(&mut self) -> bool {
    let pending_irq = self.cpu_bus.pending_irq;
    self.cpu_bus.pending_irq = false;
    pending_irq
  }
}
