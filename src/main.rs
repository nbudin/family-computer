mod cartridge;
mod cpu;
mod ines_rom;
mod instructions;
mod machine;
mod memory_map;
mod ppu;

use std::{io::Error, path::Path};

use ines_rom::INESRom;
use machine::Machine;

fn main() -> Result<(), Error> {
  let rom = INESRom::from_file(&Path::new("smb.nes"))?;
  println!("Using mapper ID {}", rom.mapper_id);
  let mut machine = Machine::from_rom(rom);
  machine.reset();

  loop {
    machine.step();
  }
}
