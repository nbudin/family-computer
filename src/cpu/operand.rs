use crate::{bus::Bus, nes::NES};

#[derive(Debug, Clone)]
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
  pub fn get_addr(&self, nes: &mut NES) -> (u16, bool) {
    let mut page_boundary_crossed = false;
    let result_addr = match self {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(nes.cpu.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(nes.cpu.y.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(nes.cpu.x));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::AbsoluteY(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(nes.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::Indirect(addr) => {
        let low = nes.cpu_bus_mut().read(*addr);
        let high = nes
          .cpu_bus_mut()
          .read((*addr & 0xff00) + u16::from(u8::try_from(*addr & 0xff).unwrap().wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectX(addr) => {
        let addr_location = nes.cpu.x.wrapping_add(*addr);
        let low = nes.cpu_bus_mut().read(u16::from(addr_location));
        let high = nes
          .cpu_bus_mut()
          .read(u16::from(addr_location.wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectY(zp_addr) => {
        let low = nes.cpu_bus_mut().read(u16::from(*zp_addr));
        let high = nes.cpu_bus_mut().read(u16::from(zp_addr.wrapping_add(1)));
        let addr = (u16::from(high) << 8) + u16::from(low);
        let new_addr = addr.wrapping_add(u16::from(nes.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      _ => {
        panic!("{:?} is not an address", self)
      }
    };

    (result_addr, page_boundary_crossed)
  }

  pub fn get_addr_readonly(&self, nes: &NES) -> (u16, bool) {
    let mut page_boundary_crossed = false;
    let result_addr = match self {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(nes.cpu.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(nes.cpu.y.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(nes.cpu.x));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::AbsoluteY(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(nes.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::Indirect(addr) => {
        let low = nes.cpu_bus().read_readonly(*addr);
        let high = nes.cpu_bus().read_readonly(
          (*addr & 0xff00) + u16::from(u8::try_from(*addr & 0xff).unwrap().wrapping_add(1)),
        );
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectX(addr) => {
        let addr_location = nes.cpu.x.wrapping_add(*addr);
        let low = nes.cpu_bus().read_readonly(u16::from(addr_location));
        let high = nes
          .cpu_bus()
          .read_readonly(u16::from(addr_location.wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectY(zp_addr) => {
        let low = nes.cpu_bus().read_readonly(u16::from(*zp_addr));
        let high = nes
          .cpu_bus()
          .read_readonly(u16::from(zp_addr.wrapping_add(1)));
        let addr = (u16::from(high) << 8) + u16::from(low);
        let new_addr = addr.wrapping_add(u16::from(nes.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      _ => {
        panic!("{:?} is not an address", self)
      }
    };

    (result_addr, page_boundary_crossed)
  }

  pub fn eval(&self, nes: &mut NES) -> (u8, bool) {
    match self {
      Operand::Accumulator => (nes.cpu.a, false),
      Operand::Immediate(value) => (*value, false),
      _ => {
        let (addr, page_boundary_crossed) = self.get_addr(nes);
        (nes.cpu_bus_mut().read(addr), page_boundary_crossed)
      }
    }
  }

  pub fn eval_readonly(&self, nes: &NES) -> (u8, bool) {
    match self {
      Operand::Accumulator => (nes.cpu.a, false),
      Operand::Immediate(value) => (*value, false),
      _ => {
        let (addr, page_boundary_crossed) = self.get_addr_readonly(nes);
        (nes.cpu_bus().read_readonly(addr), page_boundary_crossed)
      }
    }
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    match self {
      Operand::Accumulator => vec![],
      Operand::Immediate(value) => vec![*value],
      Operand::Absolute(addr)
      | Operand::AbsoluteX(addr)
      | Operand::AbsoluteY(addr)
      | Operand::Indirect(addr) => {
        vec![(*addr & 0xff) as u8, (*addr >> 8) as u8]
      }
      Operand::ZeroPage(zp_addr) | Operand::ZeroPageX(zp_addr) | Operand::ZeroPageY(zp_addr) => {
        vec![*zp_addr]
      }
      Operand::IndirectX(offset) | Operand::IndirectY(offset) => vec![*offset],
      Operand::Relative(offset) => vec![*offset as u8],
    }
  }
}
