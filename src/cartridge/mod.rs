use self::nrom::NROM;
use crate::ines_rom::INESRom;
use std::fmt::Debug;

mod nrom;

pub trait CartridgeState {}

pub trait Cartridge: Debug {
  fn from_ines_rom(rom: INESRom) -> Self
  where
    Self: Sized;
  fn get_mem(&self, addr: u16) -> u8;
  fn set_mem(&mut self, addr: u16, value: u8);
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
