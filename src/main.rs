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

use std::{
  env,
  io::BufWriter,
  path::Path,
  sync::{Arc, RwLock},
};

use iced::{Application, Settings};
use ines_rom::INESRom;
use machine::Machine;

use crate::gui::{EmulatorUI, EmulatorUIFlags};

pub fn main() -> Result<(), iced::Error> {
  let args = env::args().into_iter().collect::<Vec<_>>();
  let Some(rom_path) = args.get(1).map(Path::new) else {
    println!("Please specify a ROM path");
    return Ok(());
  };

  println!("Loading {}", rom_path.display());

  let rom = INESRom::from_file(&rom_path).unwrap();
  println!("Using mapper ID {}", rom.mapper_id);
  let mut machine = Machine::from_rom(rom);

  let stdout = std::io::stdout();

  if !env::var("DISASSEMBLE").unwrap_or_default().is_empty() {
    let disassembly_writer = BufWriter::new(stdout);
    machine.disassembly_writer = Some(Arc::new(RwLock::new(disassembly_writer)));
  }

  EmulatorUI::run(Settings::with_flags(EmulatorUIFlags::new(machine)))
}
