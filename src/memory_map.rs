use crate::{cpu::CPU, machine::MachineState, ppu::PPURegister};

impl CPU {
  pub fn get_mem(&self, addr: u16, state: &mut MachineState) -> u8 {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      state.work_ram[usize::from(actual_address)]
    } else if addr < 0x4000 {
      state.ppu_state.read_bus(PPURegister::from_address(addr))
    } else if addr < 0x4018 {
      // TODO APU and I/O registers
      0
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      0
    } else {
      state.cartridge.get_mem(addr)
    }
  }

  pub fn set_mem(&mut self, addr: u16, value: u8, state: &mut MachineState) {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      state.work_ram[usize::from(actual_address)] = value;
    } else if addr < 0x4000 {
      state
        .ppu_state
        .write_bus(PPURegister::from_address(addr), value)
    } else if addr < 0x4018 {
      // TODO APU and I/O registers
      ()
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      ()
    } else {
      state.cartridge.set_mem(addr, value)
    }
  }
}
