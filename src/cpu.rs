use crate::{instructions::Instruction, machine::MachineState};

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

#[derive(Debug)]
pub struct CPU {
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
}

impl CPU {
  pub fn new() -> Self {
    Self {
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
    }
  }

  pub fn reset(&mut self, state: &mut MachineState) {
    let low = self.get_mem(0xfffc, state);
    let high = self.get_mem(0xfffd, state);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);
    self.set_pc(&Operand::Absolute(reset_vector))
  }

  fn operand_to_addr(&self, op: &Operand, state: &mut MachineState) -> u16 {
    match op {
      Operand::ZeroPage(addr) => u16::from(*addr),
      Operand::ZeroPageX(addr) => u16::from(self.x.wrapping_add(*addr)),
      Operand::ZeroPageY(addr) => u16::from(self.x.wrapping_add(*addr)),
      Operand::Absolute(addr) => *addr,
      Operand::AbsoluteX(addr) => *addr + u16::from(self.x),
      Operand::AbsoluteY(addr) => *addr + u16::from(self.y),
      Operand::Indirect(addr) => {
        let low = self.get_mem(*addr, state);
        let high = self.get_mem(*addr + 1, state);
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectX(addr) => {
        let addr_location = self.x.wrapping_add(*addr);
        let low = self.get_mem(u16::from(addr_location), state);
        let high = self.get_mem(u16::from(addr_location.wrapping_add(1)), state);
        (u16::from(high) << 8) + u16::from(low)
      }
      Operand::IndirectY(addr) => {
        let low = self.get_mem(u16::from(*addr), state);
        let high = self.get_mem(u16::from(addr.wrapping_add(1)), state);
        (u16::from(high) << 8) + u16::from(low)
      }
      _ => {
        panic!("{:?} is not an address", op)
      }
    }
  }

  pub fn eval_operand(&self, op: &Operand, state: &mut MachineState) -> u8 {
    match op {
      Operand::Accumulator => self.a,
      Operand::Immediate(value) => *value,
      _ => self.get_mem(self.operand_to_addr(op, state), state),
    }
  }

  pub fn set_operand(&mut self, op: &Operand, value: u8, state: &mut MachineState) {
    self.set_mem(self.operand_to_addr(op, state), value, state);
  }

  pub fn set_pc(&mut self, addr: &Operand) {
    match addr {
      Operand::Absolute(addr) => {
        self.pc = *addr;
      }
      Operand::Relative(offset) => {
        (self.pc, _) = self.pc.overflowing_add_signed(i16::from(*offset));
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  fn push_stack(&mut self, value: u8, state: &mut MachineState) {
    self.set_mem(u16::from(self.s) + 0x100, value, state);
    self.s -= 1;
  }

  fn pull_stack(&mut self, state: &mut MachineState) -> u8 {
    self.s += 1;
    self.get_mem(u16::from(self.s) + 0x100, state)
  }

  pub fn step(&mut self, state: &mut MachineState) {
    let instruction = self.load_instruction(state);
    println!("{:?}", instruction);

    match instruction {
      Instruction::AND(op) => {
        let value = self.eval_operand(&op, state);
        self.a = self.a & value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::BCC(addr) => {
        if !self.carry_flag {
          self.set_pc(&addr);
        }
      }

      Instruction::BCS(addr) => {
        if self.carry_flag {
          self.set_pc(&addr);
        }
      }

      Instruction::BEQ(addr) => {
        if self.zero_flag {
          self.set_pc(&addr);
        }
      }

      Instruction::BIT(addr) => {
        let value = self.eval_operand(&addr, state);
        self.zero_flag = (value & self.a) == 0;
        self.overflow_flag = (value & (1 << 6)) > 0;
        self.negative_flag = (value & (1 << 7)) > 0;
      }

      Instruction::BMI(addr) => {
        if self.negative_flag {
          self.set_pc(&addr);
        }
      }

      Instruction::BNE(addr) => {
        if !self.zero_flag {
          self.set_pc(&addr);
        }
      }

      Instruction::BPL(addr) => {
        if !self.negative_flag {
          self.set_pc(&addr)
        }
      }

      Instruction::BRK => {
        // TODO: Interrupt request
        self.break_flag = true;
      }

      Instruction::BVC(addr) => {
        if !self.overflow_flag {
          self.set_pc(&addr);
        }
      }

      Instruction::BVS(addr) => {
        if self.overflow_flag {
          self.set_pc(&addr);
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
        let value = self.eval_operand(&op, state);
        self.carry_flag = self.a >= value;
        self.zero_flag = self.a == value;
        self.negative_flag = (self.a.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::CPX(op) => {
        let value = self.eval_operand(&op, state);
        self.carry_flag = self.x >= value;
        self.zero_flag = self.x == value;
        self.negative_flag = (self.x.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::CPY(op) => {
        let value = self.eval_operand(&op, state);
        self.carry_flag = self.y >= value;
        self.zero_flag = self.y == value;
        self.negative_flag = (self.y.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::DEC(op) => {
        let value = self.eval_operand(&op, state).wrapping_sub(1);
        self.set_operand(&op, value, state);
        self.zero_flag = value == 0;
        self.negative_flag = (value & 0b10000000) > 0;
      }

      Instruction::DEX => {
        self.x = self.x.wrapping_sub(1);
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & 0b10000000) > 0;
      }

      Instruction::DEY => {
        self.y = self.y.wrapping_sub(1);
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & 0b10000000) > 0;
      }

      Instruction::INC(op) => {
        let value = self.eval_operand(&op, state).wrapping_add(1);
        self.set_operand(&op, value, state);
        self.zero_flag = value == 0;
        self.negative_flag = (value & 0b10000000) > 0;
      }

      Instruction::INX => {
        self.x = self.x.wrapping_add(1);
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & 0b10000000) > 0;
      }

      Instruction::INY => {
        self.y = self.y.wrapping_add(1);
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & 0b10000000) > 0;
      }

      Instruction::LDA(addr) => {
        self.a = self.eval_operand(&addr, state);
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & 0b10000000) > 0;
      }

      Instruction::LDX(addr) => {
        self.x = self.eval_operand(&addr, state);
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & 0b10000000) > 0;
      }

      Instruction::LDY(addr) => {
        self.y = self.eval_operand(&addr, state);
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
        let value = self.eval_operand(&op, state);
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
