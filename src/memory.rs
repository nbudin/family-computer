use crate::cpu::Operand;

pub struct Memory {
  work_ram: [u8; 2048],
  ppu_registers: [u8; 8],
  apu_io_registers: [u8; 24],
}

impl Memory {
  pub fn new() -> Self {
    Self {
      work_ram: [0; 2048],
      ppu_registers: [0; 8],
      apu_io_registers: [0; 24],
    }
  }

  fn get_virtual(&self, addr: u16) -> u8 {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram[usize::from(actual_address)]
    } else if addr < 0x4000 {
      let actual_address = (addr - 0x2000) % 8;
      self.ppu_registers[usize::from(actual_address)]
    } else if addr < 0x4018 {
      let actual_address = addr - 0x4000;
      self.apu_io_registers[usize::from(actual_address)]
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      0
    } else {
      // TODO: cartridge space
      0
    }
  }

  fn set_virtual(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      self.work_ram[usize::from(actual_address)] = value;
    } else if addr < 0x4000 {
      let actual_address = (addr - 0x2000) % 8;
      self.ppu_registers[usize::from(actual_address)] = value;
    } else if addr < 0x4018 {
      let actual_address = addr - 0x4000;
      self.apu_io_registers[usize::from(actual_address)] = value;
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      ()
    } else {
      // TODO: cartridge space
      ()
    }
  }

  pub fn get(&self, addr: &Operand) -> u8 {
    match addr {
      Operand::Immediate(value) => *value,
      Operand::Absolute(addr) => self.get_virtual(*addr),
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  pub fn set(&mut self, addr: &Operand, value: u8) {
    match addr {
      Operand::Absolute(addr) => self.set_virtual(*addr, value),
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }
}
