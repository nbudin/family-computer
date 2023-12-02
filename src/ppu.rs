use crate::{cpu::Operand, memory::Memory};

pub enum PPURegister {
  PPUCTRL,
  PPUMASK,
  PPUSTATUS,
  OAMADDR,
  OAMDATA,
  PPUSCROLL,
  PPUADDR,
  PPUDATA,
  OAMDMA,
}

impl PPURegister {
  pub fn address(&self) -> Operand {
    match self {
      PPURegister::PPUCTRL => Operand::Absolute(0x2000),
      PPURegister::PPUMASK => Operand::Absolute(0x2001),
      PPURegister::PPUSTATUS => Operand::Absolute(0x2002),
      PPURegister::OAMADDR => Operand::Absolute(0x2003),
      PPURegister::OAMDATA => Operand::Absolute(0x2004),
      PPURegister::PPUSCROLL => Operand::Absolute(0x2005),
      PPURegister::PPUADDR => Operand::Absolute(0x2006),
      PPURegister::PPUDATA => Operand::Absolute(0x2007),
      PPURegister::OAMDMA => Operand::Absolute(0x4014),
    }
  }
}

pub struct PPU;

impl PPU {
  pub fn get_register(register: PPURegister, memory: &Memory) -> u8 {
    memory.get(&register.address())
  }

  pub fn set_register(register: PPURegister, memory: &mut Memory, value: u8) {
    memory.set(&register.address(), value)
  }
}
