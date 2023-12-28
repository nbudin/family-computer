use std::{env, fmt::Debug};

use crate::{
  cartridge::{load_cartridge, BoxCartridge},
  controller::Controller,
  cpu::{ExecutedInstruction, CPU},
  gfx::crt_screen::PIXEL_BUFFER_SIZE,
  ines_rom::INESRom,
  ppu::{PPURegister, PPU},
};

pub type WorkRAM = [u8; 2048];

pub struct Machine {
  pub work_ram: WorkRAM,
  pub cartridge: BoxCartridge,
  pub cpu: CPU,
  pub ppu: PPU,
  pub controllers: [Controller; 2],
  pub cycle_count: u64,
  pub last_executed_instruction: Option<ExecutedInstruction>,
}

impl Debug for Machine {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Machine")
      .field("cartridge", &self.cartridge)
      .field("cpu_state", &self.cpu)
      .field("ppu_state", &self.ppu)
      .field("controllers", &self.controllers)
      .field("cycle_count", &self.cycle_count)
      .field("last_executed_instruction", &self.last_executed_instruction)
      .finish_non_exhaustive()
  }
}

impl Clone for Machine {
  fn clone(&self) -> Self {
    Self {
      work_ram: self.work_ram.clone(),
      cartridge: dyn_clone::clone_box(&*self.cartridge),
      cpu: self.cpu.clone(),
      ppu: self.ppu.clone(),
      controllers: self.controllers.clone(),
      cycle_count: self.cycle_count.clone(),
      last_executed_instruction: None,
    }
  }
}

impl Machine {
  pub fn from_rom(rom: INESRom) -> Self {
    Self {
      work_ram: [0; 2048],
      cartridge: load_cartridge(rom),
      cpu: CPU::new(),
      ppu: PPU::new(),
      controllers: [Controller::new(), Controller::new()],
      cycle_count: 0,
      last_executed_instruction: None,
    }
  }

  pub fn execute_frame(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    loop {
      self.tick(pixbuf);

      if self.ppu.cycle == 0 && self.ppu.scanline == 0 {
        break;
      }
    }
  }

  pub fn tick_cpu(&mut self) {
    let executed_instruction = CPU::tick(self);
    self.cycle_count += 1;

    if let Some(instruction) = executed_instruction {
      self.last_executed_instruction = Some(instruction);
    }
  }

  pub fn tick_ppu(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    let nmi_set = PPU::tick(self, pixbuf);

    if nmi_set {
      self.nmi();
    }
  }

  pub fn tick(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    if self.cpu.wait_cycles == 0 && !env::var("DISASSEMBLE").unwrap_or_default().is_empty() {
      if let Some(executed_instruction) = &self.last_executed_instruction {
        println!("{}", executed_instruction.disassemble());
      }
    }

    self.tick_cpu();
    self.tick_ppu(pixbuf);
    self.tick_ppu(pixbuf);
    self.tick_ppu(pixbuf);
  }

  pub fn nmi(&mut self) {
    self.cpu.nmi_set = true;
  }

  pub fn reset(&mut self) {
    CPU::reset(self);
  }

  pub fn get_cpu_mem_readonly(&self, addr: u16) -> u8 {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram[usize::from(actual_address)]
    } else if addr < 0x4000 {
      PPU::read_bus_readonly(self, PPURegister::from_address(addr))
    } else if addr < 0x4016 {
      // TODO APU registers
      0
    } else if addr < 0x4018 {
      let controller = &self.controllers[addr as usize - 0x4016];
      controller.read_readonly()
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      0
    } else {
      self.cartridge.get_cpu_mem(addr)
    }
  }

  pub fn get_cpu_mem(&mut self, addr: u16) -> u8 {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram[usize::from(actual_address)]
    } else if addr < 0x4000 {
      PPU::read_bus(self, PPURegister::from_address(addr))
    } else if addr < 0x4016 {
      // TODO APU registers
      0
    } else if addr < 0x4018 {
      let controller = &mut self.controllers[addr as usize - 0x4016];
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
      PPU::write_bus(self, PPURegister::from_address(addr), value);
    } else if addr < 0x4016 {
      // TODO APU registers
      ()
    } else if addr < 0x4018 {
      let controller_index = addr as usize - 0x4016;
      let controller = &mut self.controllers[controller_index];
      controller.poll();
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      ()
    } else {
      self.cartridge.set_cpu_mem(addr, value)
    }
  }
}

#[cfg(test)]
mod tests {
  use similar_asserts::assert_eq;
  use std::io::{BufReader, BufWriter, Write};

  pub use super::*;

  #[test]
  fn nestest_smoke_test() {
    let nestest_data = include_bytes!("../smoketest/nestest.nes");
    let expected_log = include_str!("../smoketest/nestest-good.log");
    let rom = INESRom::from_reader(&mut BufReader::new(&nestest_data[..])).unwrap();

    let mut machine = Machine::from_rom(rom);
    machine.cycle_count = 7;
    machine.ppu.cycle = 21;
    machine.cpu.pc = 0xc000;

    let mut fake_pixbuf = [0; PIXEL_BUFFER_SIZE];
    let disasm_bytes: Vec<u8> = Vec::with_capacity(1 * 1024 * 1024);
    let mut disasm_writer = BufWriter::new(disasm_bytes);

    // weird PPU behavior tests start here and I'm not sure those are valid
    while machine.cycle_count < 26518 {
      machine.tick(&mut fake_pixbuf);

      if machine.cpu.wait_cycles == 0 {
        if let Some(instruction) = &machine.last_executed_instruction {
          disasm_writer
            .write_fmt(format_args!("{}\r\n", instruction.disassemble()))
            .unwrap();
        }
      }
    }

    disasm_writer.flush().unwrap();
    let disasm: String = String::from_utf8(disasm_writer.into_inner().unwrap()).unwrap();

    assert_eq!(disasm.split("\r\n").count(), 8980);
    for (disasm_line, expected_line) in disasm.split("\r\n").zip(expected_log.split("\r\n")) {
      if !disasm_line.is_empty() {
        assert_eq!(disasm_line, expected_line);
      }
    }
  }
}
