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

use fruitbasket::FruitApp;
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

  #[cfg(target_os = "macos")]
  let app = {
    let mut trampoline = fruitbasket::Trampoline::new(
      "Family Computer",
      "family-computer",
      "com.natbudin.family-computer",
    );

    let app = trampoline.build(fruitbasket::InstallDir::Temp).unwrap();
    app.set_activation_policy(fruitbasket::ActivationPolicy::Regular);
    app
  };

  println!("Loading {}", rom_path.display());

  let rom = INESRom::from_file(&rom_path).unwrap();
  // let rom = INESRom::from_reader(&mut include_bytes!("../dk.nes").as_slice()).unwrap();
  println!("Using mapper ID {}", rom.mapper_id);

  let mut flags = EmulatorUIFlags::new(Box::new(NESEmulatorBuilder::new(rom)));
  #[cfg(target_os = "macos")]
  flags.set_app(app);

  EmulatorUI::run(Settings::with_flags(flags))?;

  #[cfg(target_os = "macos")]
  FruitApp::terminate(0);

  Ok(())
}
