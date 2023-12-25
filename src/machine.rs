use std::{rc::Rc, sync::RwLock};

use crate::{
  cartridge::{load_cartridge, BoxCartridge},
  cpu::CPU,
  gfx::gfx_state::GfxState,
  ines_rom::INESRom,
  ppu::{PPURegister, PPU},
};

pub type WorkRAM = [u8; 2048];

pub struct Machine {
  pub work_ram: Rc<RwLock<WorkRAM>>,
  pub cartridge: Rc<RwLock<BoxCartridge>>,
  pub cpu_state: Rc<RwLock<CPU>>,
  pub ppu_state: Rc<RwLock<PPU>>,
}

impl Machine {
  pub fn from_rom(rom: INESRom) -> Self {
    Self {
      work_ram: Rc::new(RwLock::new([0; 2048])),
      cartridge: Rc::new(RwLock::new(load_cartridge(rom))),
      cpu_state: Rc::new(RwLock::new(CPU::new())),
      ppu_state: Rc::new(RwLock::new(PPU::new())),
    }
  }

  pub fn step(&mut self, gfx_state: &mut GfxState) {
    loop {
      {
        let mut cpu_state = (*self.cpu_state).write().unwrap();
        cpu_state.tick(&self);
      }

      {
        let mut ppu_state = (*self.ppu_state).write().unwrap();
        ppu_state.tick(&self, gfx_state);
        ppu_state.tick(&self, gfx_state);
        ppu_state.tick(&self, gfx_state);

        if ppu_state.x == 0 && ppu_state.y == 0 {
          break;
        }
      }
    }
  }

  pub fn nmi(&self) {
    let mut cpu_state = (*self.cpu_state).write().unwrap();
    cpu_state.nmi_set = true;
  }

  pub fn reset(&self) {
    let mut cpu_state = (*self.cpu_state).write().unwrap();
    cpu_state.reset(self);
  }

  pub fn get_cpu_mem(&self, addr: u16) -> u8 {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram.read().unwrap()[usize::from(actual_address)]
    } else if addr < 0x4000 {
      let mut ppu_state = (*self.ppu_state).write().unwrap();
      ppu_state.read_bus(PPURegister::from_address(addr))
    } else if addr < 0x4018 {
      // TODO APU and I/O registers
      0
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      0
    } else {
      self.cartridge.read().unwrap().get_cpu_mem(addr)
    }
  }

  pub fn set_cpu_mem(&self, addr: u16, value: u8) {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram.write().unwrap()[usize::from(actual_address)] = value;
    } else if addr < 0x4000 {
      let mut ppu_state = (*self.ppu_state).write().unwrap();
      ppu_state.write_bus(PPURegister::from_address(addr), value)
    } else if addr < 0x4018 {
      // TODO APU and I/O registers
      ()
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      ()
    } else {
      let mut cartridge = (*self.cartridge).write().unwrap();
      cartridge.set_cpu_mem(addr, value)
    }
  }
}
