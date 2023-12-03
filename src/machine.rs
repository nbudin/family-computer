use crate::{
  cartridge::{load_cartridge, BoxCartridge},
  cpu::CPU,
  ines_rom::INESRom,
  ppu::PPUState,
};

pub type WorkRAM = [u8; 2048];

#[derive(Debug)]
pub struct MachineState {
  pub work_ram: WorkRAM,
  pub cartridge: BoxCartridge,
  pub ppu_state: PPUState,
}

impl MachineState {}

pub struct Machine {
  pub cpu: CPU,
  pub state: MachineState,
}

impl Machine {
  pub fn from_rom(rom: INESRom) -> Self {
    Self {
      cpu: CPU::new(),
      state: MachineState {
        work_ram: [0; 2048],
        cartridge: load_cartridge(rom),
        ppu_state: PPUState::new(),
      },
    }
  }

  pub fn reset(&mut self) {
    self.cpu.reset(&mut self.state);
  }

  pub fn step(&mut self) {
    self.cpu.step(&mut self.state);
    self.state.ppu_state.tick();
    self.state.ppu_state.tick();
    self.state.ppu_state.tick();
  }
}
