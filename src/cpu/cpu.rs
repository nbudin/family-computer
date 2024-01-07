use std::fmt::Debug;

use bitfield_struct::bitfield;

use crate::{bus::Bus, nes::NES};

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

impl Default for CPU {
  fn default() -> Self {
    Self::new()
  }
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

  pub fn set_operand(op: &Operand, value: u8, nes: &mut NES) {
    match op {
      Operand::Accumulator => nes.cpu.a = value,
      _ => {
        let addr = op.get_addr(nes).0;
        nes.cpu_bus_mut().write(addr, value);
      }
    }
  }

  pub fn set_pc(addr: &Operand, nes: &mut NES) -> bool {
    match addr {
      Operand::Absolute(_) | Operand::Indirect(_) => {
        nes.cpu.pc = addr.get_addr(nes).0;
        false
      }
      Operand::Relative(offset) => {
        let (new_pc, _) = nes.cpu.pc.overflowing_add_signed(i16::from(*offset));
        let page_boundary_crossed = (new_pc & 0xff00) != (nes.cpu.pc & 0xff00);
        nes.cpu.pc = new_pc;
        page_boundary_crossed
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  pub fn get_stack_dump(&self, nes: &mut NES) -> Vec<u8> {
    let mut values: Vec<u8> = vec![];
    let mut addr = self.s;

    loop {
      values.push(nes.cpu_bus_mut().read(u16::from(addr) + 0x100));
      addr += 1;
      if addr > 0xfd {
        break;
      }
    }

    values
  }

  fn push_stack(value: u8, nes: &mut NES) {
    let addr = u16::from(nes.cpu.s) + 0x100;
    nes.cpu_bus_mut().write(addr, value);
    nes.cpu.s -= 1;
  }

  fn pull_stack(nes: &mut NES) -> u8 {
    nes.cpu.s += 1;
    let addr = u16::from(nes.cpu.s) + 0x100;
    nes.cpu_bus_mut().read(addr)
  }

  pub fn reset(nes: &mut NES) {
    let low = nes.cpu_bus_mut().read(0xfffc);
    let high = nes.cpu_bus_mut().read(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    CPU::set_pc(&Operand::Absolute(reset_vector), nes);

    nes.cpu.a = 0;
    nes.cpu.x = 0;
    nes.cpu.y = 0;
    nes.cpu.s = 0xfd;
    nes.cpu.p = CPUStatusRegister::from(0).with_unused(true);

    nes.cpu.wait_cycles = 7;
  }

  pub fn tick(nes: &mut NES) -> Option<ExecutedInstruction> {
    let prev_ppu_cycle = nes.ppu.cycle;
    let prev_ppu_scanline = nes.ppu.scanline;
    let prev_vram_addr: u16 = u16::from(nes.ppu.vram_addr);
    let prev_tram_addr: u16 = u16::from(nes.ppu.tram_addr);
    let prev_fine_x = nes.ppu.fine_x;
    let prev_address_latch = nes.ppu.address_latch;
    let prev_ppu_2002 = nes.cpu_bus().read_readonly(0x2002);
    let prev_ppu_2004 = nes.cpu_bus().read_readonly(0x2004);
    let prev_ppu_2007 = nes.cpu_bus().read_readonly(0x2007);
    let prev_cycle_count = nes.cpu_cycle_count;
    let prev_cpu = nes.cpu.clone();

    if nes.cpu.nmi_set {
      CPU::push_stack(u8::try_from((nes.cpu.pc & 0xff00) >> 8).unwrap(), nes);
      CPU::push_stack(u8::try_from(nes.cpu.pc & 0xff).unwrap(), nes);
      nes.cpu.p.set_break_flag(false);
      nes.cpu.p.set_interrupt_disable(true);
      nes.cpu.p.set_unused(true);
      CPU::push_stack(nes.cpu.p.into(), nes);

      let low = nes.cpu_bus_mut().read(0xfffa);
      let high = nes.cpu_bus_mut().read(0xfffb);
      let nmi_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(nmi_vector), nes);
      nes.cpu.nmi_set = false;

      nes.cpu.wait_cycles = 6;
      return None;
    }

    if nes.cpu.wait_cycles > 0 {
      nes.cpu.wait_cycles -= 1;
      return None;
    }

    if nes.cpu.irq_set && !nes.cpu.p.interrupt_disable() {
      CPU::push_stack(u8::try_from((nes.cpu.pc & 0xff00) >> 8).unwrap(), nes);
      CPU::push_stack(u8::try_from(nes.cpu.pc & 0xff).unwrap(), nes);
      CPU::push_stack(nes.cpu.p.into(), nes);
      nes.cpu.p.set_break_flag(false);
      nes.cpu.p.set_interrupt_disable(true);
      nes.cpu.p.set_unused(true);

      let low = nes.cpu_bus_mut().read(0xfffe);
      let high = nes.cpu_bus_mut().read(0xffff);
      let irq_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(irq_vector), nes);

      nes.cpu.wait_cycles = 6;
      return None;
    }

    nes.cpu.p.set_unused(true);

    let (instruction, opcode) = Instruction::load_instruction(nes);
    nes.cpu.wait_cycles = instruction.base_cycles() - 1;
    let disassembled_instruction = instruction.disassemble(&nes);
    CPU::execute_instruction(&instruction, nes, true);

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
    nes: &mut NES,
    add_page_boundary_cross_cycles: bool,
  ) {
    match &instruction {
      Instruction::ADC(op) => {
        let (value, page_boundary_crossed) = op.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }

        let result = nes.cpu.a as u16 + value as u16 + nes.cpu.p.carry_flag() as u16;
        nes.cpu.p.set_overflow_flag(
          (!(nes.cpu.a ^ value) & (nes.cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0,
        );
        nes.cpu.p.set_carry_flag(result > 255);
        nes.cpu.a = u8::try_from(result & 0xff).unwrap();
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag(nes.cpu.a & 0b10000000 > 0);
      }

      Instruction::AND(op) => {
        let (value, page_boundary_crossed) = op.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }
        nes.cpu.a &= value;
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.a & (1 << 7)) > 0);
      }

      Instruction::ASL(op) => {
        let (value, _) = op.eval(nes);
        let result = value << 1;
        CPU::set_operand(&op, result, nes);
        nes.cpu.p.set_carry_flag(value & 0b10000000 > 0);
        nes.cpu.p.set_negative_flag(result & 0b10000000 > 0);
        nes.cpu.p.set_zero_flag(result == 0);
      }

      Instruction::BCC(addr) => {
        if !nes.cpu.p.carry_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BCS(addr) => {
        if nes.cpu.p.carry_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BEQ(addr) => {
        if nes.cpu.p.zero_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BIT(addr) => {
        let (value, _) = addr.eval(nes);
        nes.cpu.p.set_zero_flag((value & nes.cpu.a) == 0);
        nes.cpu.p.set_overflow_flag((value & (1 << 6)) > 0);
        nes.cpu.p.set_negative_flag((value & (1 << 7)) > 0);
      }

      Instruction::BMI(addr) => {
        if nes.cpu.p.negative_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BNE(addr) => {
        if !nes.cpu.p.zero_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BPL(addr) => {
        if !nes.cpu.p.negative_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BRK => {
        CPU::push_stack(u8::try_from((nes.cpu.pc & 0xff00) >> 8).unwrap(), nes);
        CPU::push_stack(u8::try_from(nes.cpu.pc & 0xff).unwrap(), nes);
        nes.cpu.p.set_break_flag(true);
        CPU::push_stack(nes.cpu.p.into(), nes);

        let low = nes.cpu_bus_mut().read(0xfffe);
        let high = nes.cpu_bus_mut().read(0xffff);
        let irq_vector = (u16::from(high) << 8) + u16::from(low);

        CPU::set_pc(&Operand::Absolute(irq_vector), nes);

        nes.cpu.wait_cycles = 6;
      }

      Instruction::BVC(addr) => {
        if !nes.cpu.p.overflow_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BVS(addr) => {
        if nes.cpu.p.overflow_flag() {
          nes.cpu.wait_cycles += 1;
          if CPU::set_pc(&addr, nes) {
            nes.cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::CLC => {
        nes.cpu.p.set_carry_flag(false);
      }

      Instruction::CLD => {
        nes.cpu.p.set_decimal_flag(false);
      }

      Instruction::CLI => {
        nes.cpu.p.set_interrupt_disable(false);
      }

      Instruction::CLV => {
        nes.cpu.p.set_overflow_flag(false);
      }

      Instruction::CMP(ref op) => {
        let (value, page_boundary_crossed) = op.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }
        nes.cpu.p.set_carry_flag(nes.cpu.a >= value);
        nes.cpu.p.set_zero_flag(nes.cpu.a == value);
        nes
          .cpu
          .p
          .set_negative_flag((nes.cpu.a.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::CPX(op) => {
        let (value, _) = op.eval(nes);
        let x = nes.cpu.x;
        nes.cpu.p.set_carry_flag(x >= value);
        nes.cpu.p.set_zero_flag(x == value);
        nes
          .cpu
          .p
          .set_negative_flag((x.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::CPY(op) => {
        let (value, _) = op.eval(nes);
        let y = nes.cpu.y;
        nes.cpu.p.set_carry_flag(y >= value);
        nes.cpu.p.set_zero_flag(y == value);
        nes
          .cpu
          .p
          .set_negative_flag((y.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::DCP(op) => {
        CPU::execute_instruction(&Instruction::DEC(op.clone()), nes, false);
        CPU::execute_instruction(&Instruction::CMP(op.clone()), nes, false);
      }

      Instruction::DEC(op) => {
        let value = op.eval(nes).0.wrapping_sub(1);
        CPU::set_operand(&op, value, nes);
        nes.cpu.p.set_zero_flag(value == 0);
        nes.cpu.p.set_negative_flag((value & 0b10000000) > 0);
      }

      Instruction::DEX => {
        nes.cpu.x = nes.cpu.x.wrapping_sub(1);

        let x = nes.cpu.x;
        nes.cpu.p.set_zero_flag(x == 0);
        nes.cpu.p.set_negative_flag((x & 0b10000000) > 0);
      }

      Instruction::DEY => {
        nes.cpu.y = nes.cpu.y.wrapping_sub(1);

        let y = nes.cpu.y;
        nes.cpu.p.set_zero_flag(y == 0);
        nes.cpu.p.set_negative_flag((y & 0b10000000) > 0);
      }

      Instruction::EOR(op) => {
        let (value, page_boundary_crossed) = op.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }

        nes.cpu.a ^= value;
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag(nes.cpu.a & 0b10000000 > 0);
      }

      Instruction::INC(op) => {
        let value = op.eval(nes).0.wrapping_add(1);
        CPU::set_operand(&op, value, nes);
        nes.cpu.p.set_zero_flag(value == 0);
        nes.cpu.p.set_negative_flag((value & 0b10000000) > 0);
      }

      Instruction::INX => {
        nes.cpu.x = nes.cpu.x.wrapping_add(1);

        let x = nes.cpu.x;
        nes.cpu.p.set_zero_flag(x == 0);
        nes.cpu.p.set_negative_flag((x & 0b10000000) > 0);
      }

      Instruction::INY => {
        nes.cpu.y = nes.cpu.y.wrapping_add(1);
        nes.cpu.p.set_zero_flag(nes.cpu.y == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.y & 0b10000000) > 0);
      }

      Instruction::ISB(op) => {
        CPU::execute_instruction(&Instruction::INC(op.clone()), nes, false);
        CPU::execute_instruction(&Instruction::SBC(op.clone()), nes, false);
      }

      Instruction::JMP(addr) => {
        CPU::set_pc(&addr, nes);
      }

      Instruction::JSR(addr) => {
        let return_point = nes.cpu.pc - 1;
        let low: u8 = (return_point & 0xff).try_into().unwrap();
        let high: u8 = (return_point >> 8).try_into().unwrap();
        CPU::push_stack(high, nes);
        CPU::push_stack(low, nes);
        CPU::set_pc(&addr, nes);
      }

      Instruction::LAX(addr) => {
        CPU::execute_instruction(&Instruction::LDA(addr.clone()), nes, true);
        CPU::execute_instruction(&Instruction::TAX, nes, false);
      }

      Instruction::LDA(ref addr) => {
        let (value, page_boundary_crossed) = addr.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }
        nes.cpu.a = value;
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.a & 0b10000000) > 0);
      }

      Instruction::LDX(addr) => {
        let (value, page_boundary_crossed) = addr.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }
        nes.cpu.x = value;
        nes.cpu.p.set_zero_flag(nes.cpu.x == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.x & 0b10000000) > 0);
      }

      Instruction::LDY(addr) => {
        let (value, page_boundary_crossed) = addr.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }
        nes.cpu.y = value;
        nes.cpu.p.set_zero_flag(nes.cpu.y == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.y & 0b10000000) > 0);
      }

      Instruction::LSR(op) => {
        let (value, _) = op.eval(nes);
        let result = value >> 1;
        CPU::set_operand(&op, result, nes);
        nes.cpu.p.set_carry_flag(value & 0b1 == 1);
        nes.cpu.p.set_zero_flag(result == 0);
        nes.cpu.p.set_negative_flag(false); // always false because we always put a 0 into bit 7
      }

      Instruction::NOP => {}

      Instruction::ORA(op) => {
        let (value, page_boundary_crossed) = op.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }
        nes.cpu.a = nes.cpu.a | value;
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.a & (1 << 7)) > 0);
      }

      Instruction::PHA => {
        CPU::push_stack(nes.cpu.a, nes);
      }

      Instruction::PHP => {
        let prev_break_flag = nes.cpu.p.break_flag();
        nes.cpu.p.set_break_flag(true);
        CPU::push_stack(nes.cpu.p.into(), nes);
        nes.cpu.p.set_break_flag(prev_break_flag);
      }

      Instruction::PLA => {
        nes.cpu.a = CPU::pull_stack(nes);
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.a & 0b10000000) > 0);
      }

      Instruction::PLP => {
        let prev_break_flag = nes.cpu.p.break_flag();
        let value = CPU::pull_stack(nes);
        nes.cpu.p = value.into();
        nes.cpu.p.set_break_flag(prev_break_flag);
        nes.cpu.p.set_unused(true);
      }

      Instruction::RLA(op) => {
        CPU::execute_instruction(&Instruction::ROL(op.clone()), nes, false);
        CPU::execute_instruction(&Instruction::AND(op.clone()), nes, false);
      }

      Instruction::RRA(op) => {
        CPU::execute_instruction(&Instruction::ROR(op.clone()), nes, false);
        CPU::execute_instruction(&Instruction::ADC(op.clone()), nes, false);
      }

      Instruction::ROL(op) => {
        let (value, _) = op.eval(nes);
        let result = value << 1 | (nes.cpu.p.carry_flag() as u8);
        CPU::set_operand(&op, result, nes);
        nes.cpu.p.set_carry_flag(value & 0b10000000 > 0);
        nes.cpu.p.set_negative_flag(result & 0b10000000 > 0);
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
      }

      Instruction::ROR(op) => {
        let (value, _) = op.eval(nes);
        let result = value >> 1 | ((nes.cpu.p.carry_flag() as u8) << 7);
        CPU::set_operand(&op, result, nes);
        nes.cpu.p.set_carry_flag(value & 0b1 > 0);
        nes.cpu.p.set_negative_flag(result & 0b10000000 > 0);
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
      }

      Instruction::RTI => {
        let status = CPU::pull_stack(nes);
        nes.cpu.p = status.into();
        nes.cpu.p.set_unused(true);
        let low = CPU::pull_stack(nes);
        let high = CPU::pull_stack(nes);
        CPU::set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low)),
          nes,
        );
      }

      Instruction::RTS => {
        let low = CPU::pull_stack(nes);
        let high = CPU::pull_stack(nes);
        CPU::set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low) + 1),
          nes,
        );
      }

      Instruction::SAX(addr) => {
        CPU::set_operand(&addr, nes.cpu.a & nes.cpu.x, nes);
      }

      Instruction::SBC(op) => {
        let (value, page_boundary_crossed) = op.eval(nes);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          nes.cpu.wait_cycles += 1;
        }

        // invert the bottom 8 bits and then do addition as in ADC
        let value = value ^ 0xff;

        let result = nes.cpu.a as u16 + value as u16 + nes.cpu.p.carry_flag() as u16;
        nes.cpu.p.set_overflow_flag(
          (!(nes.cpu.a ^ value) & (nes.cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0,
        );
        nes.cpu.p.set_carry_flag(result > 255);
        nes.cpu.a = u8::try_from(result & 0xff).unwrap();
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag(nes.cpu.a & 0b10000000 > 0);
      }

      Instruction::SEC => {
        nes.cpu.p.set_carry_flag(true);
      }

      Instruction::SED => {
        nes.cpu.p.set_decimal_flag(true);
      }

      Instruction::SEI => {
        nes.cpu.p.set_interrupt_disable(true);
      }

      Instruction::SLO(op) => {
        CPU::execute_instruction(&Instruction::ASL(op.clone()), nes, false);
        CPU::execute_instruction(&Instruction::ORA(op.clone()), nes, false);
      }

      Instruction::SRE(op) => {
        CPU::execute_instruction(&Instruction::LSR(op.clone()), nes, false);
        CPU::execute_instruction(&Instruction::EOR(op.clone()), nes, false);
      }

      Instruction::STA(addr) => {
        CPU::set_operand(&addr, nes.cpu.a, nes);
      }

      Instruction::STX(addr) => {
        CPU::set_operand(&addr, nes.cpu.x, nes);
      }

      Instruction::STY(addr) => {
        CPU::set_operand(&addr, nes.cpu.y, nes);
      }

      Instruction::TAX => {
        nes.cpu.x = nes.cpu.a;
        nes.cpu.p.set_zero_flag(nes.cpu.x == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.x & (1 << 7)) > 0);
      }

      Instruction::TAY => {
        nes.cpu.y = nes.cpu.a;
        nes.cpu.p.set_zero_flag(nes.cpu.y == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.y & (1 << 7)) > 0);
      }

      Instruction::TSX => {
        nes.cpu.x = nes.cpu.s;
        nes.cpu.p.set_zero_flag(nes.cpu.x == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.x & (1 << 7)) > 0);
      }

      Instruction::TXA => {
        nes.cpu.a = nes.cpu.x;
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.a & (1 << 7)) > 0);
      }

      Instruction::TXS => {
        nes.cpu.s = nes.cpu.x;
      }

      Instruction::TYA => {
        nes.cpu.a = nes.cpu.y;
        nes.cpu.p.set_zero_flag(nes.cpu.a == 0);
        nes.cpu.p.set_negative_flag((nes.cpu.a & (1 << 7)) > 0);
      }

      Instruction::Illegal(instruction, op) => {
        match **instruction {
          Instruction::NOP => match op {
            Some(op) => {
              let (_addr, page_boundary_crossed) = op.eval(nes);
              if page_boundary_crossed && add_page_boundary_cross_cycles {
                nes.cpu.wait_cycles += 1;
              }
            }
            _ => {}
          },
          _ => {}
        }
        CPU::execute_instruction(instruction, nes, false)
      }
    }
  }
}
