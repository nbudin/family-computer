mod apu;
mod audio;
mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod gui;
mod nes;
mod ppu;

use std::{env, path::Path};

use iced::{Application, Settings};

use crate::{
  emulator::NESEmulatorBuilder,
  gui::{EmulatorUI, EmulatorUIFlags},
  nes::INESRom,
};

pub fn main() -> Result<(), iced::Error> {
  let args = env::args().into_iter().collect::<Vec<_>>();
  let Some(rom_path) = args.get(1).map(Path::new) else {
    println!("Please specify a ROM path");
    return Ok(());
  };

  println!("Loading {}", rom_path.display());

  let rom = INESRom::from_file(&rom_path).unwrap();
  // let rom = INESRom::from_reader(&mut include_bytes!("../dk.nes").as_slice()).unwrap();
  println!("Using mapper ID {}", rom.mapper_id);

  EmulatorUI::run(Settings::with_flags(EmulatorUIFlags::new(Box::new(
    NESEmulatorBuilder::new(rom),
  ))))
}
