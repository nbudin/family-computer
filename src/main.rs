mod audio;
mod bus;
mod bus_interceptor;
mod cartridge;
pub mod controller;
mod cpu;
mod dma;
mod emulator;
mod gui;
mod ines_rom;
mod machine;
mod ppu;
pub mod rw_handle;

use std::{env, path::Path};

use iced::{Application, Settings};
use ines_rom::INESRom;

use crate::{
  audio::audio_test,
  emulator::NESEmulatorBuilder,
  gui::{EmulatorUI, EmulatorUIFlags},
};

pub fn main() -> Result<(), iced::Error> {
  let args = env::args().into_iter().collect::<Vec<_>>();
  let Some(rom_path) = args.get(1).map(Path::new) else {
    println!("Please specify a ROM path");
    return Ok(());
  };

  audio_test().unwrap();

  println!("Loading {}", rom_path.display());

  let rom = INESRom::from_file(&rom_path).unwrap();
  println!("Using mapper ID {}", rom.mapper_id);

  EmulatorUI::run(Settings::with_flags(EmulatorUIFlags::new(Box::new(
    NESEmulatorBuilder::new(rom),
  ))))
}
