use dyn_clone::DynClone;

use self::nrom::NROM;
use crate::ines_rom::INESRom;
use std::fmt::Debug;

mod nrom;

pub trait CartridgeState {}

#[derive(Debug, Clone, Copy)]
pub enum CartridgeMirroring {
  HORIZONTAL,
  VERTICAL,
}

pub trait Cartridge: Debug + DynClone {
  fn from_ines_rom(rom: INESRom) -> Self
  where
    Self: Sized;
  fn get_cpu_mem(&self, addr: u16) -> u8;
  fn set_cpu_mem(&mut self, addr: u16, value: u8);
  fn get_ppu_mem(&self, addr: u16) -> Option<u8>;
  fn set_ppu_mem(&mut self, addr: u16, value: u8) -> bool;
  fn get_mirroring(&self) -> CartridgeMirroring;
}

pub type BoxCartridge = Box<dyn Cartridge>;

pub fn load_cartridge(rom: INESRom) -> BoxCartridge {
  match rom.mapper_id {
    0 => Box::new(NROM::from_ines_rom(rom)),
    _ => {
      panic!("Unsupported mapper: {}", rom.mapper_id);
    }
  }
}
