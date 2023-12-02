mod cpu;
mod ines_rom;
mod machine;
mod memory;
mod ppu;

use std::{io::Error, path::Path};

use ines_rom::INESRom;
use machine::Machine;

fn main() -> Result<(), Error> {
  let rom = INESRom::from_file(&Path::new("smb.nes"))?;
  let mut machine = Machine::from_rom(&rom);

  loop {
    machine.step();
  }
}
