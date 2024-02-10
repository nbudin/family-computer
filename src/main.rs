mod apu;
mod audio;
mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod gui;
mod nes;
mod ppu;

use std::{env, path::PathBuf, str::FromStr};

use iced::{Application, Settings};

use crate::{
  emulator::NESEmulatorBuilder,
  gui::{EmulatorUI, EmulatorUIFlags},
  nes::INESRom,
};

pub fn main() -> Result<(), iced::Error> {
  if env::var("SMOL_THREADS").is_err() {
    env::set_var("SMOL_THREADS", "4");
  }

  let args = env::args().collect::<Vec<_>>();
  let rom_path = match args
    .get(1)
    .map(|arg| PathBuf::from_str(arg.as_str()).unwrap())
  {
    Some(rom_path) => rom_path,
    None => native_dialog::FileDialog::new()
      .add_filter("iNES ROM", &["nes"])
      .set_title("Choose a NES ROM file to run")
      .show_open_single_file()
      .unwrap()
      .unwrap(),
  };

  println!("Loading {}", rom_path.display());

  let rom = INESRom::from_file(&rom_path).unwrap();
  println!("Using mapper ID {}", rom.mapper_id);

  let mut settings =
    Settings::with_flags(EmulatorUIFlags::new(Box::new(NESEmulatorBuilder::new(rom))));
  settings.exit_on_close_request = false;

  EmulatorUI::run(settings)
}
