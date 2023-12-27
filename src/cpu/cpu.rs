use std::fmt::Debug;

use crate::machine::Machine;

use super::{Instruction, LoadInstruction, Operand};

#[derive(Debug, Clone)]
pub struct CPU {
  pub wait_cycles: u8,
  pub pc: u16,
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub s: u8,

  pub negative_flag: bool,
  pub overflow_flag: bool,
  pub unused_flag: bool,
  pub break_flag: bool,
  pub decimal_flag: bool,
  pub interrupt_flag: bool,
  pub zero_flag: bool,
  pub carry_flag: bool,

  pub nmi_set: bool,
  pub irq_set: bool,
}

#[derive(Clone)]
pub struct ExecutedInstruction {
  pub instruction: Instruction,
  pub opcode: u8,
  pub prev_state: Box<Machine>,
}

impl Debug for ExecutedInstruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExecutedInstruction")
      .field("instruction", &self.instruction)
      .field("opcode", &self.opcode)
      .finish_non_exhaustive()
  }
}

impl ExecutedInstruction {
  pub fn disassemble(&self) -> String {
    let prev_cpu = &self.prev_state.cpu_state;
    let prev_ppu = &self.prev_state.ppu_state;

    format!(
      "{:04X}  {:02X} {:6}{}{:32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
      prev_cpu.pc,
      self.opcode,
      self
        .instruction
        .operand()
        .map(|op| op
          .to_bytes()
          .into_iter()
          .map(|byte| format!("{:02X}", byte))
          .collect::<Vec<_>>()
          .join(" "))
        .unwrap_or_default(),
      if matches!(self.instruction, Instruction::Illegal(_, _)) {
        "*"
      } else {
        " "
      },
      self.instruction.disassemble(&prev_cpu, &mut self.prev_state.clone()),
      prev_cpu.a,
      prev_cpu.x,
      prev_cpu.y,
      prev_cpu.get_status_register(),
      prev_cpu.s,
      prev_ppu.scanline,
      prev_ppu.cycle,
      self.prev_state.cycle_count
    )
  }
}

impl LoadInstruction for CPU {
  fn get_pc(&self) -> u16 {
    self.pc
  }

  fn inc_pc(&mut self) {
    self.pc += 1;
  }
}

impl CPU {
  pub fn new() -> Self {
    Self {
      wait_cycles: 0,
      interrupt_flag: true,
      unused_flag: true,
      carry_flag: false,
      decimal_flag: false,
      overflow_flag: false,
      negative_flag: false,
      break_flag: false,
      zero_flag: false,
      pc: 0xc000,
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
      + (if self.unused_flag { 1 << 5 } else { 0 })
      + (if self.break_flag { 1 << 4 } else { 0 })
      + (if self.decimal_flag { 1 << 3 } else { 0 })
      + (if self.interrupt_flag { 1 << 2 } else { 0 })
      + (if self.zero_flag { 1 << 1 } else { 0 })
      + (if self.carry_flag { 1 } else { 0 })
  }

  pub fn set_status_register(&mut self, value: u8) {
    self.negative_flag = (value & (1 << 7)) > 0;
    self.overflow_flag = (value & (1 << 6)) > 0;
    self.unused_flag = (value & (1 << 5)) > 0;
    self.break_flag = (value & (1 << 4)) > 0;
    self.decimal_flag = (value & (1 << 3)) > 0;
    self.interrupt_flag = (value & (1 << 2)) > 0;
    self.zero_flag = (value & (1 << 1)) > 0;
    self.carry_flag = (value & 1) > 0;
  }

  pub fn set_operand(&mut self, op: &Operand, value: u8, state: &mut Machine) {
    match op {
      Operand::Accumulator => self.a = value,
      _ => {
        let addr = op.get_addr(self, state).0;
        state.set_cpu_mem(addr, value);
      }
    }
  }

  pub fn set_pc(&mut self, addr: &Operand, state: &mut Machine) -> bool {
    match addr {
      Operand::Absolute(_) | Operand::Indirect(_) => {
        self.pc = addr.get_addr(self, state).0;
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

  pub fn get_stack_dump(&self, state: &mut Machine) -> Vec<u8> {
    let mut values: Vec<u8> = vec![];
    let mut addr = self.s;

    loop {
      values.push(state.get_cpu_mem(u16::from(addr) + 0x100));
      addr += 1;
      if addr > 0xfd {
        break;
      }
    }

    values
  }

  fn push_stack(&mut self, value: u8, state: &mut Machine) {
    state.set_cpu_mem(u16::from(self.s) + 0x100, value);
    self.s -= 1;
  }

  fn pull_stack(&mut self, state: &mut Machine) -> u8 {
    self.s += 1;
    state.get_cpu_mem(u16::from(self.s) + 0x100)
  }

  pub fn reset(mut self, state: &mut Machine) -> Self {
    let low = state.get_cpu_mem(0xfffc);
    let high = state.get_cpu_mem(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    self.set_pc(&Operand::Absolute(reset_vector), state);

    self.a = 0;
    self.x = 0;
    self.y = 0;
    self.s = 0xfd;
    self.set_status_register(0);
    self.unused_flag = true;

    self.wait_cycles = 7;

    self
  }

  pub fn tick(mut self, state: &mut Machine) -> (Self, Option<ExecutedInstruction>) {
    let prev_state = state.clone();

    if self.nmi_set {
      self.push_stack(u8::try_from((self.pc & 0xff00) >> 8).unwrap(), state);
      self.push_stack(u8::try_from(self.pc & 0xff).unwrap(), state);
      self.break_flag = false;
      self.interrupt_flag = true;
      self.unused_flag = true;
      self.push_stack(self.get_status_register(), state);

      let low = state.get_cpu_mem(0xfffa);
      let high = state.get_cpu_mem(0xfffb);
      let nmi_vector = (u16::from(high) << 8) + u16::from(low);

      self.set_pc(&Operand::Absolute(nmi_vector), state);
      self.nmi_set = false;

      self.wait_cycles = 6;
      return (self, None);
    }

    if self.wait_cycles > 0 {
      self.wait_cycles -= 1;
      return (self, None);
    }

    if self.irq_set && !self.interrupt_flag {
      self.push_stack(u8::try_from((self.pc & 0xff00) >> 8).unwrap(), state);
      self.push_stack(u8::try_from(self.pc & 0xff).unwrap(), state);
      self.push_stack(self.get_status_register(), state);
      self.break_flag = false;
      self.interrupt_flag = true;
      self.unused_flag = true;

      let low = state.get_cpu_mem(0xfffe);
      let high = state.get_cpu_mem(0xffff);
      let irq_vector = (u16::from(high) << 8) + u16::from(low);

      self.set_pc(&Operand::Absolute(irq_vector), state);

      self.wait_cycles = 6;
      return (self, None);
    }

    self.unused_flag = true;

    let (instruction, opcode) = self.load_instruction(state);
    self.wait_cycles = instruction.base_cycles() - 1;
    self.execute_instruction(&instruction, state, true);

    (
      self,
      Some(ExecutedInstruction {
        instruction,
        opcode,
        prev_state: Box::new(prev_state),
      }),
    )
  }

  fn execute_instruction(
    &mut self,
    instruction: &Instruction,
    state: &mut Machine,
    add_page_boundary_cross_cycles: bool,
  ) {
    match &instruction {
      Instruction::ADC(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }

        let result = self.a as u16 + value as u16 + if self.carry_flag { 1 } else { 0 };
        self.overflow_flag = (!(self.a ^ value) & (self.a ^ ((result & 0xff) as u8))) & 0x80 > 0;
        self.carry_flag = result > 255;
        self.a = u8::try_from(result & 0xff).unwrap();
        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & 0b10000000 > 0;
      }

      Instruction::AND(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }
        self.a &= value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::ASL(op) => {
        let (value, _) = op.eval(self, state);
        let result = value << 1;
        self.set_operand(&op, result, state);
        self.carry_flag = value & 0b10000000 > 0;
        self.negative_flag = result & 0b10000000 > 0;
        self.zero_flag = result == 0;
      }

      Instruction::BCC(addr) => {
        if !self.carry_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BCS(addr) => {
        if self.carry_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BEQ(addr) => {
        if self.zero_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
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
          if self.set_pc(&addr, state) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BNE(addr) => {
        if !self.zero_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BPL(addr) => {
        if !self.negative_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BRK => {
        self.push_stack(u8::try_from((self.pc & 0xff00) >> 8).unwrap(), state);
        self.push_stack(u8::try_from(self.pc & 0xff).unwrap(), state);
        self.break_flag = true;
        self.push_stack(self.get_status_register(), state);

        let low = state.get_cpu_mem(0xfffe);
        let high = state.get_cpu_mem(0xffff);
        let irq_vector = (u16::from(high) << 8) + u16::from(low);

        self.set_pc(&Operand::Absolute(irq_vector), state);

        self.wait_cycles = 6;
      }

      Instruction::BVC(addr) => {
        if !self.overflow_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
            self.wait_cycles += 1;
          }
        }
      }

      Instruction::BVS(addr) => {
        if self.overflow_flag {
          self.wait_cycles += 1;
          if self.set_pc(&addr, state) {
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

      Instruction::CMP(ref op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
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

      Instruction::DCP(op) => {
        self.execute_instruction(&Instruction::DEC(op.clone()), state, false);
        self.execute_instruction(&Instruction::CMP(op.clone()), state, false);
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

      Instruction::EOR(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }

        self.a ^= value;
        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & 0b10000000 > 0;
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

      Instruction::ISB(op) => {
        self.execute_instruction(&Instruction::INC(op.clone()), state, false);
        self.execute_instruction(&Instruction::SBC(op.clone()), state, false);
      }

      Instruction::JMP(addr) => {
        self.set_pc(&addr, state);
      }

      Instruction::JSR(addr) => {
        let return_point = self.pc - 1;
        let low: u8 = (return_point & 0xff).try_into().unwrap();
        let high: u8 = (return_point >> 8).try_into().unwrap();
        self.push_stack(high, state);
        self.push_stack(low, state);
        self.set_pc(&addr, state);
      }

      Instruction::LAX(addr) => {
        self.execute_instruction(&Instruction::LDA(addr.clone()), state, true);
        self.execute_instruction(&Instruction::TAX, state, false);
      }

      Instruction::LDA(ref addr) => {
        let (value, page_boundary_crossed) = addr.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }
        self.a = value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & 0b10000000) > 0;
      }

      Instruction::LDX(addr) => {
        let (value, page_boundary_crossed) = addr.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }
        self.x = value;
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & 0b10000000) > 0;
      }

      Instruction::LDY(addr) => {
        let (value, page_boundary_crossed) = addr.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }
        self.y = value;
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & 0b10000000) > 0;
      }

      Instruction::LSR(op) => {
        let (value, _) = op.eval(self, state);
        let result = value >> 1;
        self.set_operand(&op, result, state);
        self.carry_flag = value & 0b1 == 1;
        self.zero_flag = result == 0;
        self.negative_flag = false; // always false because we always put a 0 into bit 7
      }

      Instruction::NOP => {}

      Instruction::ORA(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }
        self.a = self.a | value;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::PHA => {
        self.push_stack(self.a, state);
      }

      Instruction::PHP => {
        let prev_break_flag = self.break_flag;
        self.break_flag = true;
        self.push_stack(self.get_status_register(), state);
        self.break_flag = prev_break_flag;
      }

      Instruction::PLA => {
        self.a = self.pull_stack(state);
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & 0b10000000) > 0;
      }

      Instruction::PLP => {
        let prev_break_flag = self.break_flag;
        let value = self.pull_stack(state);
        self.set_status_register(value);
        self.break_flag = prev_break_flag;
        self.unused_flag = true;
      }

      Instruction::RLA(op) => {
        self.execute_instruction(&Instruction::ROL(op.clone()), state, false);
        self.execute_instruction(&Instruction::AND(op.clone()), state, false);
      }

      Instruction::RRA(op) => {
        self.execute_instruction(&Instruction::ROR(op.clone()), state, false);
        self.execute_instruction(&Instruction::ADC(op.clone()), state, false);
      }

      Instruction::ROL(op) => {
        let (value, _) = op.eval(self, state);
        let result = value << 1 | (if self.carry_flag { 1 } else { 0 });
        self.set_operand(&op, result, state);
        self.carry_flag = value & 0b10000000 > 0;
        self.negative_flag = result & 0b10000000 > 0;
        self.zero_flag = self.a == 0;
      }

      Instruction::ROR(op) => {
        let (value, _) = op.eval(self, state);
        let result = value >> 1 | (if self.carry_flag { 0b10000000 } else { 0 });
        self.set_operand(&op, result, state);
        self.carry_flag = value & 0b1 > 0;
        self.negative_flag = result & 0b10000000 > 0;
        self.zero_flag = self.a == 0;
      }

      Instruction::RTI => {
        let status = self.pull_stack(state);
        self.set_status_register(status);
        self.unused_flag = true;
        let low = self.pull_stack(state);
        let high = self.pull_stack(state);
        self.set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low)),
          state,
        );
      }

      Instruction::RTS => {
        let low = self.pull_stack(state);
        let high = self.pull_stack(state);
        self.set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low) + 1),
          state,
        );
      }

      Instruction::SAX(addr) => {
        self.set_operand(&addr, self.a & self.x, state);
      }

      Instruction::SBC(op) => {
        let (value, page_boundary_crossed) = op.eval(self, state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          self.wait_cycles += 1;
        }

        // invert the bottom 8 bits and then do addition as in ADC
        let value = value ^ 0xff;

        let result = self.a as u16 + value as u16 + if self.carry_flag { 1 } else { 0 };
        self.overflow_flag = (!(self.a ^ value) & (self.a ^ ((result & 0xff) as u8))) & 0x80 > 0;
        self.carry_flag = result > 255;
        self.a = u8::try_from(result & 0xff).unwrap();
        self.zero_flag = self.a == 0;
        self.negative_flag = self.a & 0b10000000 > 0;
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

      Instruction::SLO(op) => {
        self.execute_instruction(&Instruction::ASL(op.clone()), state, false);
        self.execute_instruction(&Instruction::ORA(op.clone()), state, false);
      }

      Instruction::SRE(op) => {
        self.execute_instruction(&Instruction::LSR(op.clone()), state, false);
        self.execute_instruction(&Instruction::EOR(op.clone()), state, false);
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
        self.negative_flag = (self.x & (1 << 7)) > 0;
      }

      Instruction::TAY => {
        self.y = self.a;
        self.zero_flag = self.y == 0;
        self.negative_flag = (self.y & (1 << 7)) > 0;
      }

      Instruction::TSX => {
        self.x = self.s;
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & (1 << 7)) > 0;
      }

      Instruction::TXA => {
        self.a = self.x;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::TXS => {
        self.s = self.x;
      }

      Instruction::TYA => {
        self.a = self.y;
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & (1 << 7)) > 0;
      }

      Instruction::Illegal(instruction, op) => {
        match **instruction {
          Instruction::NOP => match op {
            Some(op) => {
              let (_addr, page_boundary_crossed) = op.eval(self, state);
              if page_boundary_crossed && add_page_boundary_cross_cycles {
                self.wait_cycles += 1;
              }
            }
            _ => {}
          },
          _ => {}
        }
        self.execute_instruction(instruction, state, false)
      }
    }
  }
}
