use crate::machine::Machine;

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
  pub fn get_addr(&self, state: &mut Machine) -> (u16, bool) {
    let mut page_boundary_crossed = false;
    let result_addr = match self {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(state.cpu.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(state.cpu.y.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(state.cpu.x));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::AbsoluteY(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(state.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::Indirect(addr) => {
        let low = state.get_cpu_mem(*addr);
        let high = state.get_cpu_mem(
          (*addr & 0xff00) + u16::from(u8::try_from(*addr & 0xff).unwrap().wrapping_add(1)),
        );
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectX(addr) => {
        let addr_location = state.cpu.x.wrapping_add(*addr);
        let low = state.get_cpu_mem(u16::from(addr_location));
        let high = state.get_cpu_mem(u16::from(addr_location.wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectY(zp_addr) => {
        let low = state.get_cpu_mem(u16::from(*zp_addr));
        let high = state.get_cpu_mem(u16::from(zp_addr.wrapping_add(1)));
        let addr = (u16::from(high) << 8) + u16::from(low);
        let new_addr = addr.wrapping_add(u16::from(state.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      _ => {
        panic!("{:?} is not an address", self)
      }
    };

    (result_addr, page_boundary_crossed)
  }

  pub fn get_addr_readonly(&self, state: &Machine) -> (u16, bool) {
    let mut page_boundary_crossed = false;
    let result_addr = match self {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(state.cpu.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(state.cpu.y.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(state.cpu.x));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::AbsoluteY(addr) => {
        let new_addr = (*addr).wrapping_add(u16::from(state.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      Operand::Indirect(addr) => {
        let low = state.get_cpu_mem_readonly(*addr);
        let high = state.get_cpu_mem_readonly(
          (*addr & 0xff00) + u16::from(u8::try_from(*addr & 0xff).unwrap().wrapping_add(1)),
        );
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectX(addr) => {
        let addr_location = state.cpu.x.wrapping_add(*addr);
        let low = state.get_cpu_mem_readonly(u16::from(addr_location));
        let high = state.get_cpu_mem_readonly(u16::from(addr_location.wrapping_add(1)));
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectY(zp_addr) => {
        let low = state.get_cpu_mem_readonly(u16::from(*zp_addr));
        let high = state.get_cpu_mem_readonly(u16::from(zp_addr.wrapping_add(1)));
        let addr = (u16::from(high) << 8) + u16::from(low);
        let new_addr = addr.wrapping_add(u16::from(state.cpu.y));
        page_boundary_crossed = (new_addr & 0xff00) != (addr & 0xff00);
        new_addr
      }
      _ => {
        panic!("{:?} is not an address", self)
      }
    };

    (result_addr, page_boundary_crossed)
  }

  pub fn eval(&self, state: &mut Machine) -> (u8, bool) {
    match self {
      Operand::Accumulator => (state.cpu.a, false),
      Operand::Immediate(value) => (*value, false),
      _ => {
        let (addr, page_boundary_crossed) = self.get_addr(state);
        (state.get_cpu_mem(addr), page_boundary_crossed)
      }
    }
  }

  pub fn eval_readonly(&self, state: &Machine) -> (u8, bool) {
    match self {
      Operand::Accumulator => (state.cpu.a, false),
      Operand::Immediate(value) => (*value, false),
      _ => {
        let (addr, page_boundary_crossed) = self.get_addr_readonly(state);
        (state.get_cpu_mem_readonly(addr), page_boundary_crossed)
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

  pub fn disassemble(&self, state: &Machine, eval: bool) -> String {
    let operand_formatted = match self {
      Operand::Accumulator => "A".to_owned(),
      Operand::Immediate(value) => format!("#${:02X}", value),
      Operand::Absolute(addr) => format!("${:04X}", addr),
      Operand::AbsoluteX(addr) => format!("${:04X},X", addr),
      Operand::AbsoluteY(addr) => format!("${:04X},Y", addr),
      Operand::ZeroPage(zp_addr) => format!("${:02X}", zp_addr),
      Operand::ZeroPageX(zp_addr) => format!("${:02X},X", zp_addr),
      Operand::ZeroPageY(zp_addr) => format!("${:02X},Y", zp_addr),
      Operand::Indirect(addr) => format!("(${:04X})", addr),
      Operand::IndirectX(zp_addr) => format!("(${:02X},X)", zp_addr),
      Operand::IndirectY(zp_addr) => format!("(${:02X}),Y", zp_addr),
      Operand::Relative(offset) => {
        format!("${:04X}", state.cpu.pc as i32 + *offset as i32)
      }
    };

    if eval
      && !matches!(
        self,
        Operand::Immediate(_) | Operand::Accumulator | Operand::Relative(_)
      )
    {
      match self {
        Operand::AbsoluteX(_) | Operand::AbsoluteY(_) => {
          format!(
            "{} @ {:04X} = {:02X}",
            operand_formatted,
            self.get_addr_readonly(state).0,
            self.eval_readonly(state).0
          )
        }
        Operand::ZeroPageX(_) | Operand::ZeroPageY(_) => {
          format!(
            "{} @ {:02X} = {:02X}",
            operand_formatted,
            self.get_addr_readonly(state).0,
            self.eval_readonly(state).0
          )
        }
        Operand::Indirect(_) => {
          format!(
            "{} = {:04X}",
            operand_formatted,
            self.get_addr_readonly(state).0
          )
        }
        Operand::IndirectX(zp_addr) => format!(
          "{} @ {:02X} = {:04X} = {:02X}",
          operand_formatted,
          state.cpu.x.wrapping_add(*zp_addr),
          self.get_addr_readonly(state).0,
          self.eval_readonly(state).0
        ),
        Operand::IndirectY(zp_addr) => {
          let low = state.get_cpu_mem_readonly(u16::from(*zp_addr));
          let high = state.get_cpu_mem_readonly(u16::from(zp_addr.wrapping_add(1)));
          let addr = (u16::from(high) << 8) + u16::from(low);
          format!(
            "{} = {:04X} @ {:04X} = {:02X}",
            operand_formatted,
            addr,
            self.get_addr_readonly(state).0,
            self.eval_readonly(state).0
          )
        }
        _ => format!(
          "{} = {:02X}",
          operand_formatted,
          self.eval_readonly(state).0
        ),
      }
    } else {
      operand_formatted
    }
  }
}
