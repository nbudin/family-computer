use crate::{
  cpu::{self, CPUState},
  machine::MachineState,
};

#[derive(Debug)]
pub enum Operand {
  Accumulator,
  Immediate(u8),
  Absolute(u16),
  AbsoluteX(u16),
  AbsoluteY(u16),
  ZeroPage(u8),
  ZeroPageX(u8),
  ZeroPageY(u8),
  Indirect(u16),
  IndirectX(u8),
  IndirectY(u8),
  Relative(i8),
}

impl Operand {
  pub fn get_addr(&self, cpu_state: &CPUState, machine_state: &MachineState) -> u16 {
    match self {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(cpu_state.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(cpu_state.y.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => *addr + u16::from(cpu_state.x),
      Operand::AbsoluteY(addr) => *addr + u16::from(cpu_state.y),
      Operand::Indirect(addr) => {
        let low = machine_state.get_mem(*addr);
        let high = machine_state.get_mem(*addr + 1);
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectX(addr) => {
        let addr_location = cpu_state.x.wrapping_add(*addr);
        let low = machine_state.get_mem(u16::from(addr_location));
        let high = machine_state.get_mem(u16::from(addr_location.wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectY(addr) => {
        let low = machine_state.get_mem(u16::from(*addr));
        let high = machine_state.get_mem(u16::from(addr.wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low) + u16::from(cpu_state.y)
      }
      _ => {
        panic!("{:?} is not an address", self)
      }
    }
  }

  pub fn eval(&self, cpu_state: &CPUState, machine_state: &MachineState) -> u8 {
    match self {
      Operand::Accumulator => cpu_state.a,
      Operand::Immediate(value) => *value,
      _ => {
        let addr = self.get_addr(cpu_state, machine_state);
        machine_state.get_mem(addr)
      }
    }
  }
}
