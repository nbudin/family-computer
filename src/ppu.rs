use glyphon::cosmic_text::rustybuzz::ttf_parser::kern;

use crate::{machine::Machine, operand::Operand};

#[derive(Debug)]
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
  pub fn from_address(addr: u16) -> Self {
    match addr % 8 {
      0 => Self::PPUCTRL,
      1 => Self::PPUMASK,
      2 => Self::PPUSTATUS,
      3 => Self::OAMADDR,
      4 => Self::OAMDATA,
      5 => Self::PPUSCROLL,
      6 => Self::PPUADDR,
      7 => Self::OAMDMA,
      _ => panic!("This should never happen"),
    }
  }

  pub fn address(&self) -> Operand {
    match self {
      Self::PPUCTRL => Operand::Absolute(0x2000),
      Self::PPUMASK => Operand::Absolute(0x2001),
      Self::PPUSTATUS => Operand::Absolute(0x2002),
      Self::OAMADDR => Operand::Absolute(0x2003),
      Self::OAMDATA => Operand::Absolute(0x2004),
      Self::PPUSCROLL => Operand::Absolute(0x2005),
      Self::PPUADDR => Operand::Absolute(0x2006),
      Self::PPUDATA => Operand::Absolute(0x2007),
      Self::OAMDMA => Operand::Absolute(0x4014),
    }
  }
}

#[derive(Debug)]
pub struct PPU {
  pub x: u16,
  pub y: u16,
  nmi_enable: bool,
  master_slave: bool,
  sprite_height: bool,
  background_tile_select: bool,
  increment_mode: bool,
  sprite_overflow: bool,
  even_frame: bool,
  sprite0_hit: bool,
  nametable_select: u8,
  data_bus: u8,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      nmi_enable: false,
      master_slave: false,
      sprite_height: false,
      background_tile_select: false,
      increment_mode: false,
      sprite0_hit: false,
      sprite_overflow: false,
      even_frame: false,
      nametable_select: 0,
      data_bus: 0,
    }
  }

  pub fn read_bus(&mut self, register: PPURegister) -> u8 {
    match register {
      PPURegister::PPUSTATUS => {
        self.data_bus = (self.data_bus & 0b00011111)
          + (if self.y > 239 { 1 << 7 } else { 0 })
          + (if self.sprite0_hit { 1 << 6 } else { 0 })
          + (if self.sprite_overflow { 1 << 5 } else { 0 })
      }
      _ => {}
    }

    self.data_bus
  }

  pub fn write_bus(&mut self, register: PPURegister, value: u8) {
    match register {
      PPURegister::PPUCTRL => {
        self.nmi_enable = (value & (1 << 7)) > 0;
        self.master_slave = (value & (1 << 6)) > 0;
        self.sprite_height = (value & (1 << 5)) > 0;
        self.background_tile_select = (value & (1 << 4)) > 0;
        // TODO low 4 bits

        self.data_bus = value;
      }
      _ => {}
    }
  }

  pub fn tick(&mut self, machine: &Machine) {
    if self.x < 341 {
      self.x += 1;
    } else if self.y < 262 {
      self.x = 0;
      self.y += 1;
      if self.y == 240 && self.nmi_enable {
        machine.nmi();
      }
    } else {
      self.x = 0;
      self.y = 0;
    }
  }
}
