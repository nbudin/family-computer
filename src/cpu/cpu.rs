use std::fmt::Debug;

use bitfield_struct::bitfield;

use super::{CPUBusTrait, ExecutedInstruction, Instruction, Operand};

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
#[allow(clippy::upper_case_acronyms)]
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

  pub fn set_operand(op: &Operand, value: u8, cpu_bus: &mut dyn CPUBusTrait, cpu: &mut CPU) {
    match op {
      Operand::Accumulator => cpu.a = value,
      _ => {
        let addr = op.get_addr(cpu, cpu_bus).0;
        cpu_bus.write(addr, value);
      }
    }
  }

  pub fn set_pc(addr: &Operand, cpu_bus: &mut dyn CPUBusTrait, cpu: &mut CPU) -> bool {
    match addr {
      Operand::Absolute(_) | Operand::Indirect(_) => {
        cpu.pc = addr.get_addr(cpu, cpu_bus).0;
        false
      }
      Operand::Relative(offset) => {
        let (new_pc, _) = cpu.pc.overflowing_add_signed(i16::from(*offset));
        let page_boundary_crossed = (new_pc & 0xff00) != (cpu.pc & 0xff00);
        cpu.pc = new_pc;
        page_boundary_crossed
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  pub fn get_stack_dump(&self, cpu_bus: &dyn CPUBusTrait) -> Vec<u8> {
    let mut values: Vec<u8> = vec![];
    let mut addr = self.s;

    loop {
      values.push(cpu_bus.read_readonly(u16::from(addr) + 0x100));
      addr += 1;
      if addr > 0xfd {
        break;
      }
    }

    values
  }

  fn push_stack(value: u8, cpu_bus: &mut dyn CPUBusTrait, cpu: &mut CPU) {
    let addr = u16::from(cpu.s) + 0x100;
    cpu_bus.write(addr, value);
    cpu.s -= 1;
  }

  fn pull_stack(cpu_bus: &mut dyn CPUBusTrait, cpu: &mut CPU) -> u8 {
    cpu.s += 1;
    let addr = u16::from(cpu.s) + 0x100;
    cpu_bus.read(addr)
  }

  pub fn reset(cpu_bus: &mut dyn CPUBusTrait, cpu: &mut CPU) {
    let low = cpu_bus.read(0xfffc);
    let high = cpu_bus.read(0xfffd);
    let reset_vector = (u16::from(high) << 8) + u16::from(low);

    CPU::set_pc(&Operand::Absolute(reset_vector), cpu_bus, cpu);

    cpu.a = 0;
    cpu.x = 0;
    cpu.y = 0;
    cpu.s = 0xfd;
    cpu.p = CPUStatusRegister::from(0).with_unused(true);

    cpu.wait_cycles = 7;
  }

  pub fn tick(&mut self, cpu_bus: &mut dyn CPUBusTrait) -> Option<ExecutedInstruction> {
    if self.nmi_set {
      CPU::push_stack(
        u8::try_from((self.pc & 0xff00) >> 8).unwrap(),
        cpu_bus,
        self,
      );
      CPU::push_stack(u8::try_from(self.pc & 0xff).unwrap(), cpu_bus, self);
      self.p.set_break_flag(false);
      self.p.set_interrupt_disable(true);
      self.p.set_unused(true);
      CPU::push_stack(self.p.into(), cpu_bus, self);

      let low = cpu_bus.read(0xfffa);
      let high = cpu_bus.read(0xfffb);
      let nmi_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(nmi_vector), cpu_bus, self);
      self.nmi_set = false;

      self.wait_cycles = 6;
      return None;
    }

    if self.wait_cycles > 0 {
      self.wait_cycles -= 1;
      return None;
    }

    if self.irq_set && !self.p.interrupt_disable() {
      CPU::push_stack(
        u8::try_from((self.pc & 0xff00) >> 8).unwrap(),
        cpu_bus,
        self,
      );
      CPU::push_stack(u8::try_from(self.pc & 0xff).unwrap(), cpu_bus, self);
      CPU::push_stack(self.p.into(), cpu_bus, self);
      self.p.set_break_flag(false);
      self.p.set_interrupt_disable(true);
      self.p.set_unused(true);

      let low = cpu_bus.read(0xfffe);
      let high = cpu_bus.read(0xffff);
      let irq_vector = (u16::from(high) << 8) + u16::from(low);

      CPU::set_pc(&Operand::Absolute(irq_vector), cpu_bus, self);

      self.wait_cycles = 6;
      return None;
    }

    self.p.set_unused(true);

    let (instruction, opcode) = Instruction::load_instruction(cpu_bus, self);
    self.wait_cycles = instruction.base_cycles() - 1;
    let disassembled_instruction = instruction.disassemble(cpu_bus, self);
    CPU::execute_instruction(&instruction, true, cpu_bus, self);

    Some(ExecutedInstruction {
      instruction,
      opcode,
      disassembled_instruction,
    })
  }

  fn execute_instruction(
    instruction: &Instruction,
    add_page_boundary_cross_cycles: bool,
    cpu_bus: &mut dyn CPUBusTrait,
    cpu: &mut CPU,
  ) {
    match &instruction {
      Instruction::ADC(op) => {
        let (value, page_boundary_crossed) = op.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }

        let result = cpu.a as u16 + value as u16 + cpu.p.carry_flag() as u16;
        cpu
          .p
          .set_overflow_flag((!(cpu.a ^ value) & (cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0);
        cpu.p.set_carry_flag(result > 255);
        cpu.a = u8::try_from(result & 0xff).unwrap();
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag(cpu.a & 0b10000000 > 0);
      }

      Instruction::AND(op) => {
        let (value, page_boundary_crossed) = op.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }
        cpu.a &= value;
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag((cpu.a & (1 << 7)) > 0);
      }

      Instruction::ASL(op) => {
        let (value, _) = op.eval(cpu, cpu_bus);
        let result = value << 1;
        CPU::set_operand(op, result, cpu_bus, cpu);
        cpu.p.set_carry_flag(value & 0b10000000 > 0);
        cpu.p.set_negative_flag(result & 0b10000000 > 0);
        cpu.p.set_zero_flag(result == 0);
      }

      Instruction::BCC(addr) => {
        if !cpu.p.carry_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BCS(addr) => {
        if cpu.p.carry_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BEQ(addr) => {
        if cpu.p.zero_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BIT(addr) => {
        let (value, _) = addr.eval(cpu, cpu_bus);
        cpu.p.set_zero_flag((value & cpu.a) == 0);
        cpu.p.set_overflow_flag((value & (1 << 6)) > 0);
        cpu.p.set_negative_flag((value & (1 << 7)) > 0);
      }

      Instruction::BMI(addr) => {
        if cpu.p.negative_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BNE(addr) => {
        if !cpu.p.zero_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BPL(addr) => {
        if !cpu.p.negative_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BRK => {
        CPU::push_stack(u8::try_from((cpu.pc & 0xff00) >> 8).unwrap(), cpu_bus, cpu);
        CPU::push_stack(u8::try_from(cpu.pc & 0xff).unwrap(), cpu_bus, cpu);
        cpu.p.set_break_flag(true);
        CPU::push_stack(cpu.p.into(), cpu_bus, cpu);

        let low = cpu_bus.read(0xfffe);
        let high = cpu_bus.read(0xffff);
        let irq_vector = (u16::from(high) << 8) + u16::from(low);

        CPU::set_pc(&Operand::Absolute(irq_vector), cpu_bus, cpu);

        cpu.wait_cycles = 6;
      }

      Instruction::BVC(addr) => {
        if !cpu.p.overflow_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::BVS(addr) => {
        if cpu.p.overflow_flag() {
          cpu.wait_cycles += 1;
          if CPU::set_pc(addr, cpu_bus, cpu) {
            cpu.wait_cycles += 1;
          }
        }
      }

      Instruction::CLC => {
        cpu.p.set_carry_flag(false);
      }

      Instruction::CLD => {
        cpu.p.set_decimal_flag(false);
      }

      Instruction::CLI => {
        cpu.p.set_interrupt_disable(false);
      }

      Instruction::CLV => {
        cpu.p.set_overflow_flag(false);
      }

      Instruction::CMP(ref op) => {
        let (value, page_boundary_crossed) = op.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }
        cpu.p.set_carry_flag(cpu.a >= value);
        cpu.p.set_zero_flag(cpu.a == value);
        cpu
          .p
          .set_negative_flag((cpu.a.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::CPX(op) => {
        let (value, _) = op.eval(cpu, cpu_bus);
        let x = cpu.x;
        cpu.p.set_carry_flag(x >= value);
        cpu.p.set_zero_flag(x == value);
        cpu
          .p
          .set_negative_flag((x.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::CPY(op) => {
        let (value, _) = op.eval(cpu, cpu_bus);
        let y = cpu.y;
        cpu.p.set_carry_flag(y >= value);
        cpu.p.set_zero_flag(y == value);
        cpu
          .p
          .set_negative_flag((y.wrapping_sub(value) & 0b10000000) > 0);
      }

      Instruction::DCP(op) => {
        CPU::execute_instruction(&Instruction::DEC(op.clone()), false, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::CMP(op.clone()), false, cpu_bus, cpu);
      }

      Instruction::DEC(op) => {
        let value = op.eval(cpu, cpu_bus).0.wrapping_sub(1);
        CPU::set_operand(op, value, cpu_bus, cpu);
        cpu.p.set_zero_flag(value == 0);
        cpu.p.set_negative_flag((value & 0b10000000) > 0);
      }

      Instruction::DEX => {
        cpu.x = cpu.x.wrapping_sub(1);

        let x = cpu.x;
        cpu.p.set_zero_flag(x == 0);
        cpu.p.set_negative_flag((x & 0b10000000) > 0);
      }

      Instruction::DEY => {
        cpu.y = cpu.y.wrapping_sub(1);

        let y = cpu.y;
        cpu.p.set_zero_flag(y == 0);
        cpu.p.set_negative_flag((y & 0b10000000) > 0);
      }

      Instruction::EOR(op) => {
        let (value, page_boundary_crossed) = op.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }

        cpu.a ^= value;
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag(cpu.a & 0b10000000 > 0);
      }

      Instruction::INC(op) => {
        let value = op.eval(cpu, cpu_bus).0.wrapping_add(1);
        CPU::set_operand(op, value, cpu_bus, cpu);
        cpu.p.set_zero_flag(value == 0);
        cpu.p.set_negative_flag((value & 0b10000000) > 0);
      }

      Instruction::INX => {
        cpu.x = cpu.x.wrapping_add(1);

        let x = cpu.x;
        cpu.p.set_zero_flag(x == 0);
        cpu.p.set_negative_flag((x & 0b10000000) > 0);
      }

      Instruction::INY => {
        cpu.y = cpu.y.wrapping_add(1);
        cpu.p.set_zero_flag(cpu.y == 0);
        cpu.p.set_negative_flag((cpu.y & 0b10000000) > 0);
      }

      Instruction::ISB(op) => {
        CPU::execute_instruction(&Instruction::INC(op.clone()), false, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::SBC(op.clone()), false, cpu_bus, cpu);
      }

      Instruction::JMP(addr) => {
        CPU::set_pc(addr, cpu_bus, cpu);
      }

      Instruction::JSR(addr) => {
        let return_point = cpu.pc - 1;
        let low: u8 = (return_point & 0xff).try_into().unwrap();
        let high: u8 = (return_point >> 8).try_into().unwrap();
        CPU::push_stack(high, cpu_bus, cpu);
        CPU::push_stack(low, cpu_bus, cpu);
        CPU::set_pc(addr, cpu_bus, cpu);
      }

      Instruction::LAX(addr) => {
        CPU::execute_instruction(&Instruction::LDA(addr.clone()), true, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::TAX, false, cpu_bus, cpu);
      }

      Instruction::LDA(ref addr) => {
        let (value, page_boundary_crossed) = addr.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }
        cpu.a = value;
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag((cpu.a & 0b10000000) > 0);
      }

      Instruction::LDX(addr) => {
        let (value, page_boundary_crossed) = addr.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }
        cpu.x = value;
        cpu.p.set_zero_flag(cpu.x == 0);
        cpu.p.set_negative_flag((cpu.x & 0b10000000) > 0);
      }

      Instruction::LDY(addr) => {
        let (value, page_boundary_crossed) = addr.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }
        cpu.y = value;
        cpu.p.set_zero_flag(cpu.y == 0);
        cpu.p.set_negative_flag((cpu.y & 0b10000000) > 0);
      }

      Instruction::LSR(op) => {
        let (value, _) = op.eval(cpu, cpu_bus);
        let result = value >> 1;
        CPU::set_operand(op, result, cpu_bus, cpu);
        cpu.p.set_carry_flag(value & 0b1 == 1);
        cpu.p.set_zero_flag(result == 0);
        cpu.p.set_negative_flag(false); // always false because we always put a 0 into bit 7
      }

      Instruction::NOP => {}

      Instruction::ORA(op) => {
        let (value, page_boundary_crossed) = op.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }
        cpu.a |= value;
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag((cpu.a & (1 << 7)) > 0);
      }

      Instruction::PHA => {
        CPU::push_stack(cpu.a, cpu_bus, cpu);
      }

      Instruction::PHP => {
        let prev_break_flag = cpu.p.break_flag();
        cpu.p.set_break_flag(true);
        CPU::push_stack(cpu.p.into(), cpu_bus, cpu);
        cpu.p.set_break_flag(prev_break_flag);
      }

      Instruction::PLA => {
        cpu.a = CPU::pull_stack(cpu_bus, cpu);
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag((cpu.a & 0b10000000) > 0);
      }

      Instruction::PLP => {
        let prev_break_flag = cpu.p.break_flag();
        let value = CPU::pull_stack(cpu_bus, cpu);
        cpu.p = value.into();
        cpu.p.set_break_flag(prev_break_flag);
        cpu.p.set_unused(true);
      }

      Instruction::RLA(op) => {
        CPU::execute_instruction(&Instruction::ROL(op.clone()), false, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::AND(op.clone()), false, cpu_bus, cpu);
      }

      Instruction::RRA(op) => {
        CPU::execute_instruction(&Instruction::ROR(op.clone()), false, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::ADC(op.clone()), false, cpu_bus, cpu);
      }

      Instruction::ROL(op) => {
        let (value, _) = op.eval(cpu, cpu_bus);
        let result = value << 1 | (cpu.p.carry_flag() as u8);
        CPU::set_operand(op, result, cpu_bus, cpu);
        cpu.p.set_carry_flag(value & 0b10000000 > 0);
        cpu.p.set_negative_flag(result & 0b10000000 > 0);
        cpu.p.set_zero_flag(cpu.a == 0);
      }

      Instruction::ROR(op) => {
        let (value, _) = op.eval(cpu, cpu_bus);
        let result = value >> 1 | ((cpu.p.carry_flag() as u8) << 7);
        CPU::set_operand(op, result, cpu_bus, cpu);
        cpu.p.set_carry_flag(value & 0b1 > 0);
        cpu.p.set_negative_flag(result & 0b10000000 > 0);
        cpu.p.set_zero_flag(cpu.a == 0);
      }

      Instruction::RTI => {
        let status = CPU::pull_stack(cpu_bus, cpu);
        cpu.p = status.into();
        cpu.p.set_unused(true);
        let low = CPU::pull_stack(cpu_bus, cpu);
        let high = CPU::pull_stack(cpu_bus, cpu);
        CPU::set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low)),
          cpu_bus,
          cpu,
        );
      }

      Instruction::RTS => {
        let low = CPU::pull_stack(cpu_bus, cpu);
        let high = CPU::pull_stack(cpu_bus, cpu);
        CPU::set_pc(
          &Operand::Absolute((u16::from(high) << 8) + u16::from(low) + 1),
          cpu_bus,
          cpu,
        );
      }

      Instruction::SAX(addr) => {
        CPU::set_operand(addr, cpu.a & cpu.x, cpu_bus, cpu);
      }

      Instruction::SBC(op) => {
        let (value, page_boundary_crossed) = op.eval(cpu, cpu_bus);
        if page_boundary_crossed && add_page_boundary_cross_cycles {
          cpu.wait_cycles += 1;
        }

        // invert the bottom 8 bits and then do addition as in ADC
        let value = value ^ 0xff;

        let result = cpu.a as u16 + value as u16 + cpu.p.carry_flag() as u16;
        cpu
          .p
          .set_overflow_flag((!(cpu.a ^ value) & (cpu.a ^ ((result & 0xff) as u8))) & 0x80 > 0);
        cpu.p.set_carry_flag(result > 255);
        cpu.a = u8::try_from(result & 0xff).unwrap();
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag(cpu.a & 0b10000000 > 0);
      }

      Instruction::SEC => {
        cpu.p.set_carry_flag(true);
      }

      Instruction::SED => {
        cpu.p.set_decimal_flag(true);
      }

      Instruction::SEI => {
        cpu.p.set_interrupt_disable(true);
      }

      Instruction::SLO(op) => {
        CPU::execute_instruction(&Instruction::ASL(op.clone()), false, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::ORA(op.clone()), false, cpu_bus, cpu);
      }

      Instruction::SRE(op) => {
        CPU::execute_instruction(&Instruction::LSR(op.clone()), false, cpu_bus, cpu);
        CPU::execute_instruction(&Instruction::EOR(op.clone()), false, cpu_bus, cpu);
      }

      Instruction::STA(addr) => {
        CPU::set_operand(addr, cpu.a, cpu_bus, cpu);
      }

      Instruction::STX(addr) => {
        CPU::set_operand(addr, cpu.x, cpu_bus, cpu);
      }

      Instruction::STY(addr) => {
        CPU::set_operand(addr, cpu.y, cpu_bus, cpu);
      }

      Instruction::TAX => {
        cpu.x = cpu.a;
        cpu.p.set_zero_flag(cpu.x == 0);
        cpu.p.set_negative_flag((cpu.x & (1 << 7)) > 0);
      }

      Instruction::TAY => {
        cpu.y = cpu.a;
        cpu.p.set_zero_flag(cpu.y == 0);
        cpu.p.set_negative_flag((cpu.y & (1 << 7)) > 0);
      }

      Instruction::TSX => {
        cpu.x = cpu.s;
        cpu.p.set_zero_flag(cpu.x == 0);
        cpu.p.set_negative_flag((cpu.x & (1 << 7)) > 0);
      }

      Instruction::TXA => {
        cpu.a = cpu.x;
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag((cpu.a & (1 << 7)) > 0);
      }

      Instruction::TXS => {
        cpu.s = cpu.x;
      }

      Instruction::TYA => {
        cpu.a = cpu.y;
        cpu.p.set_zero_flag(cpu.a == 0);
        cpu.p.set_negative_flag((cpu.a & (1 << 7)) > 0);
      }

      Instruction::Illegal(instruction, op) => {
        match **instruction {
          Instruction::NOP => match op {
            Some(op) => {
              let (_addr, page_boundary_crossed) = op.eval(cpu, cpu_bus);
              if page_boundary_crossed && add_page_boundary_cross_cycles {
                cpu.wait_cycles += 1;
              }
            }
            _ => {}
          },
          _ => {}
        }
        CPU::execute_instruction(instruction, false, cpu_bus, cpu)
      }
    }
  }
}
