use crate::{instructions::Instruction, machine::Machine, operand::Operand};

#[derive(Debug)]
pub struct CPU {
  pub wait_cycles: u8,
  pub pc: u16,
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub s: u8,

  pub negative_flag: bool,
  pub overflow_flag: bool,
  pub break_flag: bool,
  pub decimal_flag: bool,
  pub interrupt_flag: bool,
  pub zero_flag: bool,
  pub carry_flag: bool,

  pub nmi_set: bool,
  pub irq_set: bool,
}

impl CPU {
  pub fn new() -> Self {
    Self {
      wait_cycles: 0,
      interrupt_flag: false,
      carry_flag: false,
      decimal_flag: false,
      overflow_flag: false,
      negative_flag: false,
      break_flag: false,
      zero_flag: false,
      pc: 0,
      a: 0,
      x: 0,
      y: 0,
      s: 0xfd,
      nmi_set: false,
      irq_set: false,
    }
  }

  pub fn get_status_register(&self) -> u8 {
    (if self.negative_flag { 1 << 7 } else { 0 })
      + (if self.overflow_flag { 1 << 6 } else { 0 })
      + (1 << 5)
      + (if self.break_flag { 1 << 4 } else { 0 })
      + (if self.decimal_flag { 1 << 3 } else { 0 })
      + (if self.interrupt_flag { 1 << 2 } else { 0 })
      + (if self.carry_flag { 1 } else { 0 })
  }

  pub fn set_operand(&self, op: &Operand, value: u8, state: &Machine) {
    state.set_mem(op.get_addr(self, state).0, value);
  }

  pub fn set_pc(&mut self, addr: &Operand) -> bool {
    match addr {
      Operand::Absolute(addr) => {
        self.pc = *addr;
        false
      }
      Operand::Relative(offset) => {
        let (new_pc, _) = self.pc.overflowing_add_signed(i16::from(*offset));
        let page_boundary_crossed = (new_pc & 0xff00) != (self.pc & 0xff00);
        self.pc = new_pc;
        page_boundary_crossed
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  fn push_stack(&mut self, value: u8, state: &Machine) {
    state.set_mem(u16::from(self.s) + 0x100, value);
    self.s -= 1;
  }

  fn pull_stack(&mut self, state: &Machine) -> u8 {
    self.s += 1;
    state.get_mem(u16::from(self.s) + 0x100)
  }

  pub fn reset(&mut self, state: &Machine) {
    let low = state.get_mem(0xfffc);
    let high = state.get_mem(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    self.set_pc(&Operand::Absolute(reset_vector));
  }

  pub fn tick(&mut self, state: &Machine) {
    if self.nmi_set {
      self.push_stack(u8::try_from((self.pc & 0xff00) >> 8).unwrap(), state);
      self.push_stack(u8::try_from(self.pc & 0xff).unwrap(), state);
      self.push_stack(self.get_status_register(), state);

      let low = state.get_mem(0xfffa);
      let high = state.get_mem(0xfffb);
      let nmi_vector = (u16::from(high) << 8) + u16::from(low);

      self.set_pc(&Operand::Absolute(nmi_vector));
      self.interrupt_flag = true;
      self.nmi_set = false;

      self.wait_cycles = 6;
      return;
    }

    if self.wait_cycles > 0 {
      self.wait_cycles -= 1;
      return;
    }

    let instruction = self.load_instruction(state);

    self.wait_cycles = instruction.base_cycles() - 1;

    match instruction {
      Instruction::AND(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed {
          self.wait_cycles += 1;
        }
        self.a &= value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::BCC(addr) => {
        if !self.carry_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BCS(addr) => {
        if self.carry_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BEQ(addr) => {
        if self.zero_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BIT(addr) => {
        let (value, _) = addr.eval(self, state);
        self.zero_flag = (value & self.a) == 0;
        self.overflow_flag = (value & (1 << 6)) > 0;
        self.negative_flag = (value & (1 << 7)) > 0;
      }

      Instruction::BMI(addr) => {
        if self.negative_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BNE(addr) => {
        if !self.zero_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BPL(addr) => {
        if !self.negative_flag {
          self.wait_cycles += 1;
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BRK => {
        // TODO: Interrupt request
        self.break_flag = true;
      }

      Instruction::BVC(addr) => {
        if !self.overflow_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BVS(addr) => {
        if self.overflow_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::CLC => {
        self.carry_flag = false;
      }

      Instruction::CLD => {
        self.decimal_flag = false;
      }

      Instruction::CLI => {
        self.interrupt_flag = false;
      }

      Instruction::CLV => {
        self.overflow_flag = false;
      }

      Instruction::CMP(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed {
          self.wait_cycles += 1;
        }
        self.carry_flag = self.a >= value;
        self.zero_flag = self.a == value;
        self.negative_flag = (self.a.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::CPX(op) => {
        let (value, _) = op.eval(self, state);
        let x = self.x;
        self.carry_flag = x >= value;
        self.zero_flag = x == value;
        self.negative_flag = (x.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::CPY(op) => {
        let (value, _) = op.eval(self, state);
        let y = self.y;
        self.carry_flag = y >= value;
        self.zero_flag = y == value;
        self.negative_flag = (y.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::DEC(op) => {
        let value = op.eval(self, state).0.wrapping_sub(1);
        self.set_operand(&op, value, state);
        self.zero_flag = value == 0;
        self.negative_flag = (value & 0b10000000) > 0;
      }

      Instruction::DEX => {
        self.x = self.x.wrapping_sub(1);

        let x = self.x;
        self.zero_flag = x == 0;
        self.negative_flag = (x & 0b10000000) > 0;
      }

      Instruction::DEY => {
        self.y = self.y.wrapping_sub(1);

        let y = self.y;
        self.zero_flag = y == 0;
        self.negative_flag = (y & 0b10000000) > 0;
      }

      Instruction::INC(op) => {
        let value = op.eval(self, state).0.wrapping_add(1);
        self.set_operand(&op, value, state);
        self.zero_flag = value == 0;
        self.negative_flag = (value & 0b10000000) > 0;
      }

      Instruction::INX => {
        self.x = self.x.wrapping_add(1);

        let x = self.x;
        self.zero_flag = x == 0;
        self.negative_flag = (x & 0b10000000) > 0;
      }

      Instruction::INY => {
        self.y = self.y.wrapping_add(1);
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & 0b10000000) > 0;
      }

      Instruction::LDA(addr) => {
        let (value, page_boundary_crossed) = addr.eval(self, state);
        if page_boundary_crossed {
          self.wait_cycles += 1;
        }
        self.a = value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & 0b10000000) > 0;
      }

      Instruction::LDX(addr) => {
        let (value, page_boundary_crossed) = addr.eval(self, state);
        if page_boundary_crossed {
          self.wait_cycles += 1;
        }
        self.x = value;
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & 0b10000000) > 0;
      }

      Instruction::LDY(addr) => {
        let (value, page_boundary_crossed) = addr.eval(self, state);
        if page_boundary_crossed {
          self.wait_cycles += 1;
        }
        self.y = value;
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & 0b10000000) > 0;
      }

      Instruction::JMP(addr) => {
        self.set_pc(&addr);
      }

      Instruction::JSR(addr) => {
        let low: u8 = (self.pc % 256).try_into().unwrap();
        let high: u8 = (self.pc >> 8).try_into().unwrap();
        self.push_stack(high, state);
        self.push_stack(low, state);
        self.set_pc(&addr);
      }

      Instruction::PHA => {
        self.push_stack(self.a, state);
      }

      Instruction::ORA(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed {
          self.wait_cycles += 1;
        }
        self.a = self.a | value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::PLA => {
        self.a = self.pull_stack(state);
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & 0b10000000) > 0;
      }

      Instruction::RTS => {
        let low = self.pull_stack(state);
        let high = self.pull_stack(state);
        self.set_pc(&Operand::Absolute((u16::from(high) << 8) + u16::from(low)));
      }

      Instruction::SEC => {
        self.carry_flag = true;
      }

      Instruction::SED => {
        self.decimal_flag = true;
      }

      Instruction::SEI => {
        self.interrupt_flag = true;
      }

      Instruction::STA(addr) => {
        self.set_operand(&addr, self.a, state);
      }

      Instruction::STX(addr) => {
        self.set_operand(&addr, self.x, state);
      }

      Instruction::STY(addr) => {
        self.set_operand(&addr, self.y, state);
      }

      Instruction::TAX => {
        self.x = self.a;
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & (1 << 6)) > 0;
      }

      Instruction::TAY => {
        self.y = self.a;
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & (1 << 6)) > 0;
      }

      Instruction::TSX => {
        self.x = self.s;
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & (1 << 6)) > 0;
      }

      Instruction::TXA => {
        self.a = self.x;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 6)) > 0;
      }

      Instruction::TXS => {
        self.s = self.x;
      }

      Instruction::TYA => {
        self.a = self.y;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 6)) > 0;
      }

      #[allow(unreachable_patterns)]
      _ => {
        panic!("Unknown instruction: {:?}", instruction);
      }
    }
  }
}
