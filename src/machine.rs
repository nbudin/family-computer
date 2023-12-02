use crate::{cpu::CPU, ines_rom::INESRom, memory::Memory};

pub struct Machine {
  pub cpu: CPU,
  pub memory: Memory,
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
}

impl Machine {
  pub fn from_rom(rom: &INESRom) -> Self {
    Self {
      cpu: CPU::new(),
      memory: Memory::new(),
      prg_rom: rom.prg_data.clone(),
      chr_rom: rom.chr_data.clone(),
    }
  }

  pub fn step(&mut self) {
    self.cpu.step(&self.prg_rom, &mut self.memory);
  }
}
