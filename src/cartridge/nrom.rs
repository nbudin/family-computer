use super::{Cartridge, CartridgeMirroring, CartridgeState};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct NROM {
  pub prg_rom: [u8; 32 * 1024],
  pub chr_rom: [u8; 8 * 1024],
  pub state: NROMState,
  mirroring: CartridgeMirroring,
}

impl Cartridge for NROM {
  fn from_ines_rom(rom: crate::ines_rom::INESRom) -> Self {
    let mut prg_rom: [u8; 32 * 1024] = [0; 32 * 1024];
    if !rom.prg_data.is_empty() {
      for chunk in prg_rom.chunks_exact_mut(rom.prg_data.len()) {
        chunk.copy_from_slice(&rom.prg_data);
      }
    }

    let mut chr_rom: [u8; 8 * 1024] = [0; 8 * 1024];
    if !rom.chr_data.is_empty() {
      for chunk in chr_rom.chunks_exact_mut(rom.chr_data.len()) {
        chunk.copy_from_slice(&rom.chr_data);
      }
    }

    Self {
      prg_rom,
      chr_rom,
      state: NROMState::new(),
      mirroring: if rom.vertical_mirroring {
        CartridgeMirroring::VERTICAL
      } else {
        CartridgeMirroring::HORIZONTAL
      },
    }
  }

  fn get_cpu_mem(&self, addr: u16) -> Option<u8> {
    if addr < 0x6000 {
      None
    } else if addr < 0x8000 {
      Some(self.state.prg_ram[usize::from(addr) % (8 * 1024)])
    } else {
      Some(self.prg_rom[usize::from(addr - 0x8000)])
    }
  }

  fn set_cpu_mem(&mut self, addr: u16, value: u8) -> bool {
    if addr < 0x6000 {
      false
    } else if addr < 0x8000 {
      self.state.prg_ram[usize::from(addr) % (8 * 1024)] = value;
      true
    } else {
      // can't write to rom
      true
    }
  }

  fn get_ppu_mem(&self, addr: u16) -> Option<u8> {
    if addr < 0x2000 {
      Some(self.chr_rom[usize::from(addr)])
    } else {
      None
    }
  }

  fn set_ppu_mem(&mut self, addr: u16, value: u8) -> bool {
    if addr < 0x2000 {
      self.chr_rom[usize::from(addr)] = value;
      true
    } else {
      false
    }
  }

  fn get_mirroring(&self) -> CartridgeMirroring {
    self.mirroring
  }
}
