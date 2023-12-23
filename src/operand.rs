use std::fmt::Display;

use crate::{cpu::CPU, machine::Machine};

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
  pub fn get_addr(&self, cpu_state: &CPU, machine_state: &Machine) -> (u16, bool) {
    let mut page_boundary_crossed = false;
    let result_addr = match self {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(cpu_state.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(cpu_state.y.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => {
        let new_addr = *addr + u16::from(cpu_state.x);
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::AbsoluteY(addr) => {
        let new_addr = *addr + u16::from(cpu_state.y);
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
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
      Operand::IndirectY(zp_addr) => {
        let low = machine_state.get_mem(u16::from(*zp_addr));
        let high = machine_state.get_mem(u16::from(zp_addr.wrapping_add(1)));
        let addr = (u16::from(high) << 8) + u16::from(low);
        let new_addr = addr + u16::from(cpu_state.y);
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      _ => {
        panic!("{:?} is not an address", self)
      }
    };

    (result_addr, page_boundary_crossed)
  }

  pub fn eval(&self, cpu_state: &CPU, machine_state: &Machine) -> (u8, bool) {
    match self {
      Operand::Accumulator => (cpu_state.a, false),
      Operand::Immediate(value) => (*value, false),
      _ => {
        let (addr, page_boundary_crossed) = self.get_addr(cpu_state, machine_state);
        (machine_state.get_mem(addr), page_boundary_crossed)
      }
    }
  }
}

impl Display for Operand {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Operand::Accumulator => f.write_str(""),
      Operand::Immediate(value) => f.write_fmt(format_args!("#${:02x}", value)),
      Operand::Absolute(addr) => f.write_fmt(format_args!("${:04x}", addr)),
      Operand::AbsoluteX(addr) => f.write_fmt(format_args!("${:04x},x", addr)),
      Operand::AbsoluteY(addr) => f.write_fmt(format_args!("${:04x},y", addr)),
      Operand::ZeroPage(zp_addr) => f.write_fmt(format_args!("${:02x}", zp_addr)),
      Operand::ZeroPageX(zp_addr) => f.write_fmt(format_args!("${:02x},x", zp_addr)),
      Operand::ZeroPageY(zp_addr) => f.write_fmt(format_args!("${:02x},y", zp_addr)),
      Operand::Indirect(addr) => f.write_fmt(format_args!("(${:04x})", addr)),
      Operand::IndirectX(zp_addr) => f.write_fmt(format_args!("(${:02x},x)", zp_addr)),
      Operand::IndirectY(zp_addr) => f.write_fmt(format_args!("(${:02x}),y", zp_addr)),
      Operand::Relative(offset) => f.write_fmt(format_args!("{}", offset)),
    }
  }
}
