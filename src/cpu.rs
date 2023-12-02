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
      pc: 0x8000,
      a: 0,
      x: 0,
      y: 0,
      s: 0xfd,
    }
  }

  pub fn eval_operand(&self, op: &Operand, state: &mut MachineState) -> u8 {
    match op {
      Operand::Accumulator => self.a,
      Operand::Immediate(value) => *value,
      Operand::ZeroPage(addr) => self.get_mem(u16::from(*addr), state),
      Operand::ZeroPageX(addr) => self.get_mem(u16::from(self.x.wrapping_add(*addr)), state),
      Operand::ZeroPageY(addr) => self.get_mem(u16::from(self.x.wrapping_add(*addr)), state),
      Operand::Absolute(addr) => self.get_mem(*addr, state),
      Operand::AbsoluteX(addr) => self.get_mem(*addr + u16::from(self.x), state),
      Operand::AbsoluteY(addr) => self.get_mem(*addr + u16::from(self.y), state),
      Operand::Indirect(addr) => {
        let low = self.get_mem(*addr, state);
        let high = self.get_mem(*addr + 1, state);
        let target_addr = (u16::from(high) << 8) + u16::from(low);
        self.get_mem(target_addr, state)
      }
      Operand::IndirectX(addr) => {
        let addr_location = self.x.wrapping_add(*addr);
        let low = self.get_mem(u16::from(addr_location), state);
        let high = self.get_mem(u16::from(addr_location.wrapping_add(1)), state);
        let target_addr = (u16::from(high) << 8) + u16::from(low);
        self.get_mem(target_addr, state)
      }
      Operand::IndirectY(addr) => {
        let low = self.get_mem(u16::from(*addr), state);
        let high = self.get_mem(u16::from(addr.wrapping_add(1)), state);
        let target_addr = (u16::from(high) << 8) + u16::from(low);
        self.get_mem(target_addr + u16::from(self.y), state)
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", op);
      }
    }
  }

  pub fn set_operand(&mut self, op: &Operand, value: u8, state: &mut MachineState) {
    match op {
      Operand::Absolute(addr) => self.set_mem(*addr, value, state),
      _ => {
        panic!("Unknown addressing mode: {:?}", op);
      }
    }
  }

  pub fn set_pc(&mut self, addr: &Operand) {
    match addr {
      Operand::Relative(offset) => {
        (self.pc, _) = self.pc.overflowing_add_signed(i16::from(*offset));
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  pub fn step(&mut self, state: &mut MachineState) {
    let instruction = self.load_instruction(state);
    println!("{:?}", instruction);

    match instruction {
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

      Instruction::TXS => {
        self.s = self.x;
      }

      #[allow(unreachable_patterns)]
      _ => {
        panic!("Unknown instruction: {:?}", instruction);
      }
    }
  }
}
