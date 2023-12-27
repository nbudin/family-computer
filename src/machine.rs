use std::env;

use crate::{
  cartridge::{load_cartridge, BoxCartridge},
  controller::{Controller, ControllerState},
  cpu::{ExecutedInstruction, CPU},
  gfx::crt_screen::PIXEL_BUFFER_SIZE,
  ines_rom::INESRom,
  ppu::{PPURegister, PPU},
};

pub type WorkRAM = [u8; 2048];

pub struct Machine {
  pub work_ram: WorkRAM,
  pub cartridge: BoxCartridge,
  pub cpu_state: CPU,
  pub ppu_state: PPU,
  pub controllers: [Controller; 2],
  pub cycle_count: u64,
  pub last_executed_instruction: Option<ExecutedInstruction>,
}

impl Clone for Machine {
  fn clone(&self) -> Self {
    Self {
      work_ram: self.work_ram.clone(),
      cartridge: dyn_clone::clone_box(&*self.cartridge),
      cpu_state: self.cpu_state.clone(),
      ppu_state: self.ppu_state.clone(),
      controllers: self.controllers.clone(),
      cycle_count: self.cycle_count.clone(),
      last_executed_instruction: self.last_executed_instruction.clone(),
    }
  }
}

impl Machine {
  pub fn from_rom(rom: INESRom) -> Self {
    Self {
      work_ram: [0; 2048],
      cartridge: load_cartridge(rom),
      cpu_state: CPU::new(),
      ppu_state: PPU::new(),
      controllers: [Controller::new(); 2],
      cycle_count: 0,
      last_executed_instruction: None,
    }
  }

  pub fn execute_frame(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    loop {
      self.tick(pixbuf);

      if self.ppu_state.cycle == 0 && self.ppu_state.scanline == 0 {
        break;
      }
    }
  }

  pub fn tick(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    let mut prev_state = self.clone();
    if self.cpu_state.wait_cycles == 0 && !env::var("DISASSEMBLE").unwrap_or_default().is_empty() {
      if let Some(executed_instruction) = &self.last_executed_instruction {
        println!(
          "{}",
          executed_instruction.disassemble(&mut prev_state, &self)
        );
      }
    }

    self.last_executed_instruction = {
      let (new_cpu_state, executed_instruction) = self.cpu_state.clone().tick(self);
      self.cycle_count += 1;
      self.cpu_state = new_cpu_state;

      executed_instruction
    };

    {
      let new_ppu_state = self
        .ppu_state
        .clone()
        .tick(self, pixbuf)
        .tick(self, pixbuf)
        .tick(self, pixbuf);
      self.ppu_state = new_ppu_state;
    }
  }

  pub fn nmi(&mut self) {
    self.cpu_state.nmi_set = true;
  }

  pub fn reset(&mut self) {
    let new_cpu_state = self.cpu_state.clone().reset(self);
    self.cpu_state = new_cpu_state;
  }

  pub fn get_cpu_mem(&mut self, addr: u16) -> u8 {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram[usize::from(actual_address)]
    } else if addr < 0x4000 {
      let (new_ppu_state, result) = self
        .ppu_state
        .clone()
        .read_bus(self, PPURegister::from_address(addr));
      self.ppu_state = new_ppu_state;
      result
    } else if addr < 0x4016 {
      // TODO APU registers
      0
    } else if addr < 0x4018 {
      let mut controller = self.controllers[addr as usize - 0x4016];
      controller.read()
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      0
    } else {
      self.cartridge.get_cpu_mem(addr)
    }
  }

  pub fn set_cpu_mem(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram[usize::from(actual_address)] = value;
    } else if addr < 0x4000 {
      let new_ppu_state =
        self
          .ppu_state
          .clone()
          .write_bus(self, PPURegister::from_address(addr), value);
      self.ppu_state = new_ppu_state;
    } else if addr < 0x4016 {
      // TODO APU registers
      ()
    } else if addr < 0x4018 {
      let mut controller = self.controllers[addr as usize - 0x4016];
      controller.write();
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      ()
    } else {
      self.cartridge.set_cpu_mem(addr, value)
    }
  }

  pub fn update_controller<F: FnOnce(&mut ControllerState) -> ()>(
    &self,
    controller_index: usize,
    f: F,
  ) {
    let mut controller = self.controllers[controller_index];
    controller.update(f);
  }
}
