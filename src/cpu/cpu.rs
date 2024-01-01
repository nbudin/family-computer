use std::fmt::Debug;

use bitfield_struct::bitfield;

use crate::{bus::Bus, machine::Machine};

use super::{ExecutedInstruction, Instruction, Operand};

#[bitfield(u8)]
pub struct CPUStatusRegister {
  pub carry_flag: bool,
  pub zero_flag: bool,
  pub interrupt_disable: bool,
  pub decimal_flag: bool,
  pub break_flag: bool,
  pub unused: bool,
  pub overflow_flag: bool,
  pub negative_flag: bool,
}

#[derive(Debug, Clone)]
pub struct CPU {
  pub wait_cycles: u8,
  pub pc: u16,
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub s: u8,
  pub p: CPUStatusRegister,

  pub nmi_set: bool,
  pub irq_set: bool,
}

impl CPU {
  pub fn new() -> Self {
    Self {
      wait_cycles: 0,
      p: CPUStatusRegister::from(0)
        .with_interrupt_disable(true)
        .with_unused(true),
      pc: 0xc000,
      a: 0,
      x: 0,
      y: 0,
      s: 0xfd,
      nmi_set: false,
      irq_set: false,
    }
  }

  pub fn set_operand(op: &Operand, value: u8, state: &mut Machine) {
    match op {
      Operand::Accumulator => state.cpu.a = value,
      _ => {
        let addr = op.get_addr(state).0;
        state.cpu_bus_mut().write(addr, value);
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
      values.push(state.cpu_bus_mut().read(u16::from(addr) + 0x100));
      addr += 1;
      if addr > 0xfd {
        break;
      }
    }

    values
  }

  fn push_stack(value: u8, state: &mut Machine) {
    let addr = u16::from(state.cpu.s) + 0x100;
    state.cpu_bus_mut().write(addr, value);
    state.cpu.s -= 1;
  }

  fn pull_stack(state: &mut Machine) -> u8 {
    state.cpu.s += 1;
    let addr = u16::from(state.cpu.s) + 0x100;
    state.cpu_bus_mut().read(addr)
  }

  pub fn reset(state: &mut Machine) {
    let low = state.cpu_bus_mut().read(0xfffc);
    let high = state.cpu_bus_mut().read(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    CPU::set_pc(&Operand::Absolute(reset_vector), state);

    state.cpu.a = 0;
    state.cpu.x = 0;
    state.cpu.y = 0;
    state.cpu.s = 0xfd;
    state.cpu.p = CPUStatusRegister::from(0).with_unused(true);

    state.cpu.wait_cycles = 7;
  }

  pub fn tick(state: &mut Machine) -> Option<ExecutedInstruction> {
    let prev_ppu_cycle = state.ppu.cycle;
    let prev_ppu_scanline = state.ppu.scanline;
    let prev_vram_addr: u16 = u16::from(state.ppu.vram_addr);
    let prev_tram_addr: u16 = u16::from(state.ppu.tram_addr);
    let prev_fine_x = state.ppu.fine_x;
    let prev_address_latch = state.ppu.address_latch;
    let prev_ppu_2002 = state.cpu_bus().read_readonly(0x2002);
    let prev_ppu_2004 = state.cpu_bus().read_readonly(0x2004);
    let prev_ppu_2007 = state.cpu_bus().read_readonly(0x2007);
    let prev_cycle_count = state.cpu_cycle_count;
    let prev_cpu = state.cpu.clone();

    if state.cpu.nmi_set {
      CPU::push_stack(u8::try_from((state.cpu.pc & 0xff00) >> 8).unwrap(), state);
      CPU::push_stack(u8::try_from(state.cpu.pc & 0xff).unwrap(), state);
      state.cpu.p.set_break_flag(false);
      state.cpu.p.set_interrupt_disable(true);
      state.cpu.p.set_unused(true);
      CPU::push_stack(state.cpu.p.into(), state);

      let low = state.cpu_bus_mut().read(0xfffa);
      let high = state.cpu_bus_mut().read(0xfffb);
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

    if state.cpu.irq_set && !state.cpu.p.interrupt_disable() {
      CPU::push_stack(u8::try_from((state.cpu.pc & 0xff00) >> 8).unwrap(), state);
      CPU::push_stack(u8::try_from(state.cpu.pc & 0xff).unwrap(), state);
      CPU::push_stack(state.cpu.p.into(), state);
      state.cpu.p.set_break_flag(false);
      state.cpu.p.set_interrupt_disable(true);
      state.cpu.p.set_unused(true);

      let low = state.cpu_bus_mut().read(0xfffe);
      let high = state.cpu_bus_mut().read(0xffff);
      let irq_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(irq_vector), state);

      state.cpu.wait_cycles = 6;
      return None;
    }

    state.cpu.p.set_unused(true);

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
      ppu2002: prev_ppu_2002,
      ppu2004: prev_ppu_2004,
      ppu2007: prev_ppu_2007,
      tram_addr: prev_tram_addr,
      vram_addr: prev_vram_addr,
      fine_x: prev_fine_x,
      ppu_address_latch: prev_address_latch,
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

        let result = state.cpu.a as u16 + value as u16 + state.cpu.p.carry_flag() as u16;
        state.cpu.p.set_overflow_flag(
          (!(state.cpu.a ^ value) & (state.cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0,
        );
        state.cpu.p.set_carry_flag(result > 255);
        state.cpu.a = u8::try_from(result & 0xff).unwrap();
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag(state.cpu.a & 0b10000000 > 0);
      }

      Instruction::AND(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.a &= value;
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag((state.cpu.a & (1 << 7)) > 0);
      }

      Instruction::ASL(op) => {
        let (value, _) = op.eval(state);
        let result = value << 1;
        CPU::set_operand(&op, result, state);
        state.cpu.p.set_carry_flag(value & 0b10000000 > 0);
        state.cpu.p.set_negative_flag(result & 0b10000000 > 0);
        state.cpu.p.set_zero_flag(result == 0);
      }

      Instruction::BCC(addr) => {
        if !state.cpu.p.carry_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BCS(addr) => {
        if state.cpu.p.carry_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BEQ(addr) => {
        if state.cpu.p.zero_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BIT(addr) => {
        let (value, _) = addr.eval(state);
        state.cpu.p.set_zero_flag((value & state.cpu.a) == 0);
        state.cpu.p.set_overflow_flag((value & (1 << 6)) > 0);
        state.cpu.p.set_negative_flag((value & (1 << 7)) > 0);
      }

      Instruction::BMI(addr) => {
        if state.cpu.p.negative_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BNE(addr) => {
        if !state.cpu.p.zero_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BPL(addr) => {
        if !state.cpu.p.negative_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BRK => {
        CPU::push_stack(u8::try_from((state.cpu.pc & 0xff00) >> 8).unwrap(), state);
        CPU::push_stack(u8::try_from(state.cpu.pc & 0xff).unwrap(), state);
        state.cpu.p.set_break_flag(true);
        CPU::push_stack(state.cpu.p.into(), state);

        let low = state.cpu_bus_mut().read(0xfffe);
        let high = state.cpu_bus_mut().read(0xffff);
        let irq_vector = (u16::from(high) << 8) + u16::from(low);

        CPU::set_pc(&Operand::Absolute(irq_vector), state);

        state.cpu.wait_cycles = 6;
      }

      Instruction::BVC(addr) => {
        if !state.cpu.p.overflow_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BVS(addr) => {
        if state.cpu.p.overflow_flag() {
          state.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, state) {
            state.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::CLC => {
        state.cpu.p.set_carry_flag(false);
      }

      Instruction::CLD => {
        state.cpu.p.set_decimal_flag(false);
      }

      Instruction::CLI => {
        state.cpu.p.set_interrupt_disable(false);
      }

      Instruction::CLV => {
        state.cpu.p.set_overflow_flag(false);
      }

      Instruction::CMP(ref op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.p.set_carry_flag(state.cpu.a >= value);
        state.cpu.p.set_zero_flag(state.cpu.a == value);
        state
          .cpu
          .p
          .set_negative_flag((state.cpu.a.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::CPX(op) => {
        let (value, _) = op.eval(state);
        let x = state.cpu.x;
        state.cpu.p.set_carry_flag(x >= value);
        state.cpu.p.set_zero_flag(x == value);
        state
          .cpu
          .p
          .set_negative_flag((x.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::CPY(op) => {
        let (value, _) = op.eval(state);
        let y = state.cpu.y;
        state.cpu.p.set_carry_flag(y >= value);
        state.cpu.p.set_zero_flag(y == value);
        state
          .cpu
          .p
          .set_negative_flag((y.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::DCP(op) => {
        CPU::execute_instruction(&Instruction::DEC(op.clone()), state, false);
        CPU::execute_instruction(&Instruction::CMP(op.clone()), state, false);
      }

      Instruction::DEC(op) => {
        let value = op.eval(state).0.wrapping_sub(1);
        CPU::set_operand(&op, value, state);
        state.cpu.p.set_zero_flag(value == 0);
        state.cpu.p.set_negative_flag((value & 0b10000000) > 0);
      }

      Instruction::DEX => {
        state.cpu.x = state.cpu.x.wrapping_sub(1);

        let x = state.cpu.x;
        state.cpu.p.set_zero_flag(x == 0);
        state.cpu.p.set_negative_flag((x & 0b10000000) > 0);
      }

      Instruction::DEY => {
        state.cpu.y = state.cpu.y.wrapping_sub(1);

        let y = state.cpu.y;
        state.cpu.p.set_zero_flag(y == 0);
        state.cpu.p.set_negative_flag((y & 0b10000000) > 0);
      }

      Instruction::EOR(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }

        state.cpu.a ^= value;
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag(state.cpu.a & 0b10000000 > 0);
      }

      Instruction::INC(op) => {
        let value = op.eval(state).0.wrapping_add(1);
        CPU::set_operand(&op, value, state);
        state.cpu.p.set_zero_flag(value == 0);
        state.cpu.p.set_negative_flag((value & 0b10000000) > 0);
      }

      Instruction::INX => {
        state.cpu.x = state.cpu.x.wrapping_add(1);

        let x = state.cpu.x;
        state.cpu.p.set_zero_flag(x == 0);
        state.cpu.p.set_negative_flag((x & 0b10000000) > 0);
      }

      Instruction::INY => {
        state.cpu.y = state.cpu.y.wrapping_add(1);
        state.cpu.p.set_zero_flag(state.cpu.y == 0);
        state
          .cpu
          .p
          .set_negative_flag((state.cpu.y & 0b10000000) > 0);
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
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state
          .cpu
          .p
          .set_negative_flag((state.cpu.a & 0b10000000) > 0);
      }

      Instruction::LDX(addr) => {
        let (value, page_boundary_crossed) = addr.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.x = value;
        state.cpu.p.set_zero_flag(state.cpu.x == 0);
        state
          .cpu
          .p
          .set_negative_flag((state.cpu.x & 0b10000000) > 0);
      }

      Instruction::LDY(addr) => {
        let (value, page_boundary_crossed) = addr.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.y = value;
        state.cpu.p.set_zero_flag(state.cpu.y == 0);
        state
          .cpu
          .p
          .set_negative_flag((state.cpu.y & 0b10000000) > 0);
      }

      Instruction::LSR(op) => {
        let (value, _) = op.eval(state);
        let result = value >> 1;
        CPU::set_operand(&op, result, state);
        state.cpu.p.set_carry_flag(value & 0b1 == 1);
        state.cpu.p.set_zero_flag(result == 0);
        state.cpu.p.set_negative_flag(false); // always false because we always put a 0 into bit 7
      }

      Instruction::NOP => {}

      Instruction::ORA(op) => {
        let (value, page_boundary_crossed) = op.eval(state);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          state.cpu.wait_cycles += 1;
        }
        state.cpu.a = state.cpu.a | value;
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag((state.cpu.a & (1 << 7)) > 0);
      }

      Instruction::PHA => {
        CPU::push_stack(state.cpu.a, state);
      }

      Instruction::PHP => {
        let prev_break_flag = state.cpu.p.break_flag();
        state.cpu.p.set_break_flag(true);
        CPU::push_stack(state.cpu.p.into(), state);
        state.cpu.p.set_break_flag(prev_break_flag);
      }

      Instruction::PLA => {
        state.cpu.a = CPU::pull_stack(state);
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state
          .cpu
          .p
          .set_negative_flag((state.cpu.a & 0b10000000) > 0);
      }

      Instruction::PLP => {
        let prev_break_flag = state.cpu.p.break_flag();
        let value = CPU::pull_stack(state);
        state.cpu.p = value.into();
        state.cpu.p.set_break_flag(prev_break_flag);
        state.cpu.p.set_unused(true);
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
        let result = value << 1 | (state.cpu.p.carry_flag() as u8);
        CPU::set_operand(&op, result, state);
        state.cpu.p.set_carry_flag(value & 0b10000000 > 0);
        state.cpu.p.set_negative_flag(result & 0b10000000 > 0);
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
      }

      Instruction::ROR(op) => {
        let (value, _) = op.eval(state);
        let result = value >> 1 | ((state.cpu.p.carry_flag() as u8) << 7);
        CPU::set_operand(&op, result, state);
        state.cpu.p.set_carry_flag(value & 0b1 > 0);
        state.cpu.p.set_negative_flag(result & 0b10000000 > 0);
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
      }

      Instruction::RTI => {
        let status = CPU::pull_stack(state);
        state.cpu.p = status.into();
        state.cpu.p.set_unused(true);
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

        let result = state.cpu.a as u16 + value as u16 + state.cpu.p.carry_flag() as u16;
        state.cpu.p.set_overflow_flag(
          (!(state.cpu.a ^ value) & (state.cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0,
        );
        state.cpu.p.set_carry_flag(result > 255);
        state.cpu.a = u8::try_from(result & 0xff).unwrap();
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag(state.cpu.a & 0b10000000 > 0);
      }

      Instruction::SEC => {
        state.cpu.p.set_carry_flag(true);
      }

      Instruction::SED => {
        state.cpu.p.set_decimal_flag(true);
      }

      Instruction::SEI => {
        state.cpu.p.set_interrupt_disable(true);
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
        state.cpu.p.set_zero_flag(state.cpu.x == 0);
        state.cpu.p.set_negative_flag((state.cpu.x & (1 << 7)) > 0);
      }

      Instruction::TAY => {
        state.cpu.y = state.cpu.a;
        state.cpu.p.set_zero_flag(state.cpu.y == 0);
        state.cpu.p.set_negative_flag((state.cpu.y & (1 << 7)) > 0);
      }

      Instruction::TSX => {
        state.cpu.x = state.cpu.s;
        state.cpu.p.set_zero_flag(state.cpu.x == 0);
        state.cpu.p.set_negative_flag((state.cpu.x & (1 << 7)) > 0);
      }

      Instruction::TXA => {
        state.cpu.a = state.cpu.x;
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag((state.cpu.a & (1 << 7)) > 0);
      }

      Instruction::TXS => {
        state.cpu.s = state.cpu.x;
      }

      Instruction::TYA => {
        state.cpu.a = state.cpu.y;
        state.cpu.p.set_zero_flag(state.cpu.a == 0);
        state.cpu.p.set_negative_flag((state.cpu.a & (1 << 7)) > 0);
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
