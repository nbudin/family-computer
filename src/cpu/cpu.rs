use std::fmt::Debug;

use crate::machine::Machine;

use super::{Instruction, Operand};

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
  pub disassembled_instruction: String,
  pub prev_cpu: CPU,
  pub scanline: i32,
  pub cycle: i32,
  pub cycle_count: u64,
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
    format!(
      "{:04X}  {:02X} {:6}{}{:32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
      self.prev_cpu.pc,
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
      self.disassembled_instruction,
      self.prev_cpu.a,
      self.prev_cpu.x,
      self.prev_cpu.y,
      self.prev_cpu.get_status_register(),
      self.prev_cpu.s,
      self.scanline,
      self.cycle,
      self.cycle_count
    )
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

  pub fn set_operand(op: &Operand, value: u8, state: &mut Machine) {
    match op {
      Operand::Accumulator => state.cpu.a = value,
      _ => {
        let addr = op.get_addr(state).0;
        state.set_cpu_mem(addr, value);
      }
    }
  }

  pub fn set_pc(addr: &Operand, state: &mut Machine) -> bool {
    match addr {
      Operand::Absolute(_) | Operand::Indirect(_) => {
        state.cpu.pc = addr.get_addr(state).0;
        false
      }
      Operand::Relative(offset) => {
        let (new_pc, _) = state.cpu.pc.overflowing_add_signed(i16::from(*offset));
        let page_boundary_crossed = (new_pc & 0xff00) != (state.cpu.pc & 0xff00);
        state.cpu.pc = new_pc;
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

  fn push_stack(value: u8, state: &mut Machine) {
    state.set_cpu_mem(u16::from(state.cpu.s) + 0x100, value);
    state.cpu.s -= 1;
  }

  fn pull_stack(state: &mut Machine) -> u8 {
    state.cpu.s += 1;
    state.get_cpu_mem(u16::from(state.cpu.s) + 0x100)
  }

  pub fn reset(state: &mut Machine) {
    let low = state.get_cpu_mem(0xfffc);
    let high = state.get_cpu_mem(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    CPU::set_pc(&Operand::Absolute(reset_vector), state);

    state.cpu.a = 0;
    state.cpu.x = 0;
    state.cpu.y = 0;
    state.cpu.s = 0xfd;
    state.cpu.set_status_register(0);
    state.cpu.unused_flag = true;

    state.cpu.wait_cycles = 7;
  }

  pub fn tick(state: &mut Machine) -> Option<ExecutedInstruction> {
    let prev_ppu_cycle = state.ppu.cycle;
    let prev_ppu_scanline = state.ppu.scanline;
    let prev_cycle_count = state.cycle_count;
    let prev_cpu = state.cpu.clone();

    if state.cpu.nmi_set {
      CPU::push_stack(u8::try_from((state.cpu.pc & 0xff00) >> 8).unwrap(), state);
      CPU::push_stack(u8::try_from(state.cpu.pc & 0xff).unwrap(), state);
      state.cpu.break_flag = false;
      state.cpu.interrupt_flag = true;
      state.cpu.unused_flag = true;
      CPU::push_stack(state.cpu.get_status_register(), state);

      let low = state.get_cpu_mem(0xfffa);
      let high = state.get_cpu_mem(0xfffb);
      let nmi_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(nmi_vector), state);
      state.cpu.nmi_set = false;

      state.cpu.wait_cycles = 6;
      return None;
    }

    if state.cpu.wait_cycles > 0 {
      state.cpu.wait_cycles -= 1;
      return None;
    }

    if state.cpu.irq_set && !state.cpu.interrupt_flag {
      CPU::push_stack(u8::try_from((state.cpu.pc & 0xff00) >> 8).unwrap(), state);
      CPU::push_stack(u8::try_from(state.cpu.pc & 0xff).unwrap(), state);
      CPU::push_stack(state.cpu.get_status_register(), state);
      state.cpu.break_flag = false;
      state.cpu.interrupt_flag = true;
      state.cpu.unused_flag = true;

      let low = state.get_cpu_mem(0xfffe);
      let high = state.get_cpu_mem(0xffff);
      let irq_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(irq_vector), state);

      state.cpu.wait_cycles = 6;
      return None;
    }

    state.cpu.unused_flag = true;

    let (instruction, opcode) = Instruction::load_instruction(state);
    state.cpu.wait_cycles = instruction.base_cycles() - 1;
    let disassembled_instruction = instruction.disassemble(&state);
    CPU::execute_instruction(&instruction, state, true);

    Some(ExecutedInstruction {
      instruction,
      opcode,
      cycle: prev_ppu_cycle,
      scanline: prev_ppu_scanline,
      cycle_count: prev_cycle_count,
      disassembled_instruction,
      prev_cpu,
    })
  }

  fn execute_instruction(
    instruction: &Instruction,
    state: &mut Machine,
    add_page_boundary_cross_cycles: bool,
  ) {
    match &instruction {
      Instruction::ADC(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }

        let result = state.cpu.a as u16 + value as u16 + if state.cpu.carry_flag { 1 } else { 0 };
        state.cpu.overflow_flag =
          (!(state.cpu.a ^ value) & (state.cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0;
        state.cpu.carry_flag = result > 255;
        state.cpu.a = u8::try_from(result & 0xff).unwrap();
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = state.cpu.a & 0b10000000 > 0;
      }

      Instruction::AND(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.a &= value;
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = (state.cpu.a & (1 << 7)) > 0;
      }

      Instruction::ASL(op) => {
        let (value, _) = op.eval(state);
        let result = value << 1;
        CPU::set_operand(&op, result, state);
        state.cpu.carry_flag = value & 0b10000000 > 0;
        state.cpu.negative_flag = result & 0b10000000 > 0;
        state.cpu.zero_flag = result == 0;
      }

      Instruction::BCC(addr) => {
        if !state.cpu.carry_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BCS(addr) => {
        if state.cpu.carry_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BEQ(addr) => {
        if state.cpu.zero_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BIT(addr) => {
        let (value, _) = addr.eval(state);
        state.cpu.zero_flag = (value & state.cpu.a) == 0;
        state.cpu.overflow_flag = (value & (1 << 6)) > 0;
        state.cpu.negative_flag = (value & (1 << 7)) > 0;
      }

      Instruction::BMI(addr) => {
        if state.cpu.negative_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BNE(addr) => {
        if !state.cpu.zero_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BPL(addr) => {
        if !state.cpu.negative_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BRK => {
        CPU::push_stack(u8::try_from((state.cpu.pc & 0xff00) >> 8).unwrap(), state);
        CPU::push_stack(u8::try_from(state.cpu.pc & 0xff).unwrap(), state);
        state.cpu.break_flag = true;
        CPU::push_stack(state.cpu.get_status_register(), state);

        let low = state.get_cpu_mem(0xfffe);
        let high = state.get_cpu_mem(0xffff);
        let irq_vector = (u16::from(high) << 8) + u16::from(low);

        CPU::set_pc(&Operand::Absolute(irq_vector), state);

        state.cpu.wait_cycles = 6;
      }

      Instruction::BVC(addr) => {
        if !state.cpu.overflow_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BVS(addr) => {
        if state.cpu.overflow_flag {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::CLC => {
        state.cpu.carry_flag = false;
      }

      Instruction::CLD => {
        state.cpu.decimal_flag = false;
      }

      Instruction::CLI => {
        state.cpu.interrupt_flag = false;
      }

      Instruction::CLV => {
        state.cpu.overflow_flag = false;
      }

      Instruction::CMP(ref op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.carry_flag = state.cpu.a >= value;
        state.cpu.zero_flag = state.cpu.a == value;
        state.cpu.negative_flag = (state.cpu.a.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::CPX(op) => {
        let (value, _) = op.eval(state);
        let x = state.cpu.x;
        state.cpu.carry_flag = x >= value;
        state.cpu.zero_flag = x == value;
        state.cpu.negative_flag = (x.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::CPY(op) => {
        let (value, _) = op.eval(state);
        let y = state.cpu.y;
        state.cpu.carry_flag = y >= value;
        state.cpu.zero_flag = y == value;
        state.cpu.negative_flag = (y.wrapping_sub(value) & 0b10000000) > 0;
      }

      Instruction::DCP(op) => {
        CPU::execute_instruction(&Instruction::DEC(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::CMP(op.clone()), state, false);
      }

      Instruction::DEC(op) => {
        let value = op.eval(state).0.wrapping_sub(1);
        CPU::set_operand(&op, value, state);
        state.cpu.zero_flag = value == 0;
        state.cpu.negative_flag = (value & 0b10000000) > 0;
      }

      Instruction::DEX => {
        state.cpu.x = state.cpu.x.wrapping_sub(1);

        let x = state.cpu.x;
        state.cpu.zero_flag = x == 0;
        state.cpu.negative_flag = (x & 0b10000000) > 0;
      }

      Instruction::DEY => {
        state.cpu.y = state.cpu.y.wrapping_sub(1);

        let y = state.cpu.y;
        state.cpu.zero_flag = y == 0;
        state.cpu.negative_flag = (y & 0b10000000) > 0;
      }

      Instruction::EOR(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }

        state.cpu.a ^= value;
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = state.cpu.a & 0b10000000 > 0;
      }

      Instruction::INC(op) => {
        let value = op.eval(state).0.wrapping_add(1);
        CPU::set_operand(&op, value, state);
        state.cpu.zero_flag = value == 0;
        state.cpu.negative_flag = (value & 0b10000000) > 0;
      }

      Instruction::INX => {
        state.cpu.x = state.cpu.x.wrapping_add(1);

        let x = state.cpu.x;
        state.cpu.zero_flag = x == 0;
        state.cpu.negative_flag = (x & 0b10000000) > 0;
      }

      Instruction::INY => {
        state.cpu.y = state.cpu.y.wrapping_add(1);
        state.cpu.zero_flag = state.cpu.y == 0;
        state.cpu.negative_flag = (state.cpu.y & 0b10000000) > 0;
      }

      Instruction::ISB(op) => {
        CPU::execute_instruction(&Instruction::INC(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::SBC(op.clone()), state, false);
      }

      Instruction::JMP(addr) => {
        CPU::set_pc(&addr, state);
      }

      Instruction::JSR(addr) => {
        let return_point = state.cpu.pc - 1;
        let low: u8 = (return_point & 0xff).try_into().unwrap();
        let high: u8 = (return_point >> 8).try_into().unwrap();
        CPU::push_stack(high, state);
        CPU::push_stack(low, state);
        CPU::set_pc(&addr, state);
      }

      Instruction::LAX(addr) => {
        CPU::execute_instruction(&Instruction::LDA(addr.clone()), state, true);
        CPU::execute_instruction(&Instruction::TAX, state, false);
      }

      Instruction::LDA(ref addr) => {
        let (value, page_boundary_crossed) = addr.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.a = value;
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = (state.cpu.a & 0b10000000) > 0;
      }

      Instruction::LDX(addr) => {
        let (value, page_boundary_crossed) = addr.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.x = value;
        state.cpu.zero_flag = state.cpu.x == 0;
        state.cpu.negative_flag = (state.cpu.x & 0b10000000) > 0;
      }

      Instruction::LDY(addr) => {
        let (value, page_boundary_crossed) = addr.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.y = value;
        state.cpu.zero_flag = state.cpu.y == 0;
        state.cpu.negative_flag = (state.cpu.y & 0b10000000) > 0;
      }

      Instruction::LSR(op) => {
        let (value, _) = op.eval(state);
        let result = value >> 1;
        CPU::set_operand(&op, result, state);
        state.cpu.carry_flag = value & 0b1 == 1;
        state.cpu.zero_flag = result == 0;
        state.cpu.negative_flag = false; // always false because we always put a 0 into bit 7
      }

      Instruction::NOP => {}

      Instruction::ORA(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.a = state.cpu.a | value;
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = (state.cpu.a & (1 << 7)) > 0;
      }

      Instruction::PHA => {
        CPU::push_stack(state.cpu.a, state);
      }

      Instruction::PHP => {
        let prev_break_flag = state.cpu.break_flag;
        state.cpu.break_flag = true;
        CPU::push_stack(state.cpu.get_status_register(), state);
        state.cpu.break_flag = prev_break_flag;
      }

      Instruction::PLA => {
        state.cpu.a = CPU::pull_stack(state);
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = (state.cpu.a & 0b10000000) > 0;
      }

      Instruction::PLP => {
        let prev_break_flag = state.cpu.break_flag;
        let value = CPU::pull_stack(state);
        state.cpu.set_status_register(value);
        state.cpu.break_flag = prev_break_flag;
        state.cpu.unused_flag = true;
      }

      Instruction::RLA(op) => {
        CPU::execute_instruction(&Instruction::ROL(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::AND(op.clone()), state, false);
      }

      Instruction::RRA(op) => {
        CPU::execute_instruction(&Instruction::ROR(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::ADC(op.clone()), state, false);
      }

      Instruction::ROL(op) => {
        let (value, _) = op.eval(state);
        let result = value << 1 | (if state.cpu.carry_flag { 1 } else { 0 });
        CPU::set_operand(&op, result, state);
        state.cpu.carry_flag = value & 0b10000000 > 0;
        state.cpu.negative_flag = result & 0b10000000 > 0;
        state.cpu.zero_flag = state.cpu.a == 0;
      }

      Instruction::ROR(op) => {
        let (value, _) = op.eval(state);
        let result = value >> 1 | (if state.cpu.carry_flag { 0b10000000 } else { 0 });
        CPU::set_operand(&op, result, state);
        state.cpu.carry_flag = value & 0b1 > 0;
        state.cpu.negative_flag = result & 0b10000000 > 0;
        state.cpu.zero_flag = state.cpu.a == 0;
      }

      Instruction::RTI => {
        let status = CPU::pull_stack(state);
        state.cpu.set_status_register(status);
        state.cpu.unused_flag = true;
        let low = CPU::pull_stack(state);
        let high = CPU::pull_stack(state);
        CPU::set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low)),
          state,
        );
      }

      Instruction::RTS => {
        let low = CPU::pull_stack(state);
        let high = CPU::pull_stack(state);
        CPU::set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low) + 1),
          state,
        );
      }

      Instruction::SAX(addr) => {
        CPU::set_operand(&addr, state.cpu.a & state.cpu.x, state);
      }

      Instruction::SBC(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }

        // invert the bottom 8 bits and then do addition as in ADC
        let value = value ^ 0xff;

        let result = state.cpu.a as u16 + value as u16 + if state.cpu.carry_flag { 1 } else { 0 };
        state.cpu.overflow_flag =
          (!(state.cpu.a ^ value) & (state.cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0;
        state.cpu.carry_flag = result > 255;
        state.cpu.a = u8::try_from(result & 0xff).unwrap();
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = state.cpu.a & 0b10000000 > 0;
      }

      Instruction::SEC => {
        state.cpu.carry_flag = true;
      }

      Instruction::SED => {
        state.cpu.decimal_flag = true;
      }

      Instruction::SEI => {
        state.cpu.interrupt_flag = true;
      }

      Instruction::SLO(op) => {
        CPU::execute_instruction(&Instruction::ASL(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::ORA(op.clone()), state, false);
      }

      Instruction::SRE(op) => {
        CPU::execute_instruction(&Instruction::LSR(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::EOR(op.clone()), state, false);
      }

      Instruction::STA(addr) => {
        CPU::set_operand(&addr, state.cpu.a, state);
      }

      Instruction::STX(addr) => {
        CPU::set_operand(&addr, state.cpu.x, state);
      }

      Instruction::STY(addr) => {
        CPU::set_operand(&addr, state.cpu.y, state);
      }

      Instruction::TAX => {
        state.cpu.x = state.cpu.a;
        state.cpu.zero_flag = state.cpu.x == 0;
        state.cpu.negative_flag = (state.cpu.x & (1 << 7)) > 0;
      }

      Instruction::TAY => {
        state.cpu.y = state.cpu.a;
        state.cpu.zero_flag = state.cpu.y == 0;
        state.cpu.negative_flag = (state.cpu.y & (1 << 7)) > 0;
      }

      Instruction::TSX => {
        state.cpu.x = state.cpu.s;
        state.cpu.zero_flag = state.cpu.x == 0;
        state.cpu.negative_flag = (state.cpu.x & (1 << 7)) > 0;
      }

      Instruction::TXA => {
        state.cpu.a = state.cpu.x;
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = (state.cpu.a & (1 << 7)) > 0;
      }

      Instruction::TXS => {
        state.cpu.s = state.cpu.x;
      }

      Instruction::TYA => {
        state.cpu.a = state.cpu.y;
        state.cpu.zero_flag = state.cpu.a == 0;
        state.cpu.negative_flag = (state.cpu.a & (1 << 7)) > 0;
      }

      Instruction::Illegal(instruction, op) => {
        match **instruction {
          Instruction::NOP => match op {
            Some(op) => {
              let (_addr, page_boundary_crossed) = op.eval(state);
              if page_boundary_crossed && add_page_boundary_cross_cycles {
                state.cpu.wait_cycles += 1;
              }
            }
            _ => {}
          },
          _ => {}
        }
        CPU::execute_instruction(instruction, state, false)
      }
    }
  }
}
