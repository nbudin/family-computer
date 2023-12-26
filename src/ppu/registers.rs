use bitfield_struct::bitfield;

use crate::cpu::Operand;

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
      7 => Self::PPUDATA,
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

#[bitfield(u8)]
pub struct PPUStatusRegister {
  #[bits(5)]
  _unused: usize,
  pub sprite_overflow: bool,
  pub sprite_zero_hit: bool,
  pub vertical_blank: bool,
}

#[bitfield(u8)]
pub struct PPUMaskRegister {
  pub grayscale: bool,
  pub render_background_left: bool,
  pub render_sprites_left: bool,
  pub render_background: bool,
  pub render_sprites: bool,
  pub enhance_red: bool,
  pub enhance_green: bool,
  pub enhance_blue: bool,
}

#[bitfield(u8)]
pub struct PPUControlRegister {
  pub nametable_x: bool,
  pub nametable_y: bool,
  pub increment_mode: bool,
  pub pattern_sprite: bool,
  pub pattern_background: bool,
  pub sprite_size: bool,
  pub slave_mode: bool,
  pub enable_nmi: bool,
}

#[bitfield(u16)]
pub struct PPULoopyRegister {
  #[bits(5)]
  pub coarse_x: u8,
  #[bits(5)]
  pub coarse_y: u8,
  pub nametable_x: bool,
  pub nametable_y: bool,
  #[bits(3)]
  pub fine_y: u8,
  _unused: bool,
}
