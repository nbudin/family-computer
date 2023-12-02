use super::{Cartridge, CartridgeState};

#[derive(Debug)]
pub struct NROMState {
  pub prg_ram: [u8; 8 * 1024],
}

impl NROMState {
  fn new() -> Self {
    Self {
      prg_ram: [0; 8 * 1024],
    }
  }
}

impl CartridgeState for NROMState {}

#[derive(Debug)]
pub struct NROM {
  pub prg_rom: [u8; 32 * 1024],
  pub chr_rom: [u8; 8 * 1024],
  pub state: NROMState,
}

impl Cartridge for NROM {
  fn from_ines_rom(rom: crate::ines_rom::INESRom) -> Self {
    let mut prg_rom: [u8; 32 * 1024] = [0; 32 * 1024];
    prg_rom.copy_from_slice(&rom.prg_data);

    let mut chr_rom: [u8; 8 * 1024] = [0; 8 * 1024];
    chr_rom.copy_from_slice(&rom.chr_data);

    Self {
      prg_rom,
      chr_rom,
      state: NROMState::new(),
    }
  }

  fn get_mem(&self, addr: u16) -> u8 {
    if addr < 0x8000 {
      self.state.prg_ram[usize::from(addr) % (8 * 1024)]
    } else {
      self.prg_rom[usize::from(addr - 0x8000)]
    }
  }

  fn set_mem(&mut self, addr: u16, value: u8) {
    todo!()
  }
}
