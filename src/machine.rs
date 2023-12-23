use std::{rc::Rc, sync::RwLock};

use crate::{
  cartridge::{load_cartridge, BoxCartridge},
  cpu::CPUState,
  ines_rom::INESRom,
  operand::Operand,
  ppu::{PPURegister, PPUState},
};

pub type WorkRAM = [u8; 2048];

#[derive(Debug)]
pub struct MachineState {
  pub work_ram: Rc<RwLock<WorkRAM>>,
  pub cartridge: Rc<RwLock<BoxCartridge>>,
  pub cpu_state: Rc<RwLock<CPUState>>,
  pub ppu_state: Rc<RwLock<PPUState>>,
}

impl MachineState {
  pub fn reset(&mut self) {
    let low = self.get_mem(0xfffc);
    let high = self.get_mem(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    let mut cpu_state = (*self.cpu_state).write().unwrap();
    cpu_state.set_pc(&Operand::Absolute(reset_vector));
  }

  pub fn get_mem(&self, addr: u16) -> u8 {
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
      self.cartridge.read().unwrap().get_mem(addr)
    }
  }

  pub fn set_mem(&self, addr: u16, value: u8) {
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
      cartridge.set_mem(addr, value)
    }
  }
}

pub struct Machine {
  pub state: MachineState,
}

impl Machine {
  pub fn from_rom(rom: INESRom) -> Self {
    Self {
      state: MachineState {
        work_ram: Rc::new(RwLock::new([0; 2048])),
        cartridge: Rc::new(RwLock::new(load_cartridge(rom))),
        cpu_state: Rc::new(RwLock::new(CPUState::new())),
        ppu_state: Rc::new(RwLock::new(PPUState::new())),
      },
    }
  }

  pub fn step(&mut self) {
    let mut cpu_state = (*self.state.cpu_state).write().unwrap();
    cpu_state.step(&self.state);

    let mut ppu_state = (*self.state.ppu_state).write().unwrap();
    ppu_state.tick();
    ppu_state.tick();
    ppu_state.tick();
  }

  pub fn reset(&mut self) {
    self.state.reset();
  }
}
