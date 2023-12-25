use std::fmt::Display;

use strum::IntoStaticStr;

use crate::{cpu::CPU, machine::Machine, operand::Operand};

#[derive(Debug, IntoStaticStr)]
pub enum Instruction {
  ADC(Operand),
  AND(Operand),
  ASL(Operand),
  BCC(Operand),
  BCS(Operand),
  BEQ(Operand),
  BIT(Operand),
  BMI(Operand),
  BNE(Operand),
  BPL(Operand),
  BRK,
  BVC(Operand),
  BVS(Operand),
  CLC,
  CLD,
  CLI,
  CLV,
  CMP(Operand),
  CPX(Operand),
  CPY(Operand),
  DEC(Operand),
  DEX,
  DEY,
  EOR(Operand),
  INC(Operand),
  INX,
  INY,
  JMP(Operand),
  JSR(Operand),
  LDA(Operand),
  LDX(Operand),
  LDY(Operand),
  LSR(Operand),
  NOP,
  ORA(Operand),
  PHA,
  PHP,
  PLA,
  PLP,
  ROL(Operand),
  ROR(Operand),
  RTI,
  RTS,
  SBC(Operand),
  SEC,
  SED,
  SEI,
  STA(Operand),
  STX(Operand),
  STY(Operand),
  TAX,
  TAY,
  TSX,
  TXA,
  TXS,
  TYA,
}

impl Instruction {
  pub fn base_cycles(&self) -> u8 {
    match self {
      Instruction::ADC(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for ADC: {:?}", op),
      },
      Instruction::AND(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for AND: {:?}", op),
      },
      Instruction::ASL(op) => match op {
        Operand::Accumulator => 2,
        Operand::Absolute(_) => 6,
        Operand::AbsoluteX(_) => 7,
        Operand::ZeroPage(_) => 5,
        Operand::ZeroPageX(_) => 6,
        _ => panic!("Invalid operand for ASL: {:?}", op),
      },
      Instruction::BCC(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BCC: {:?}", op),
      },
      Instruction::BCS(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BCS: {:?}", op),
      },
      Instruction::BEQ(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BEQ: {:?}", op),
      },
      Instruction::BIT(op) => match op {
        Operand::Absolute(_) => 4,
        Operand::ZeroPage(_) => 3,
        _ => panic!("Invalid operand for BIT: {:?}", op),
      },
      Instruction::BMI(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BMI: {:?}", op),
      },
      Instruction::BNE(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BNE: {:?}", op),
      },
      Instruction::BPL(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BPL: {:?}", op),
      },
      Instruction::BRK => 7,
      Instruction::BVC(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BVC: {:?}", op),
      },
      Instruction::BVS(op) => match op {
        Operand::Relative(_) => 2,
        _ => panic!("Invalid operand for BVS: {:?}", op),
      },
      Instruction::CLC => 2,
      Instruction::CLD => 2,
      Instruction::CLI => 2,
      Instruction::CLV => 2,
      Instruction::CMP(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for CMP: {:?}", op),
      },
      Instruction::CPX(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::ZeroPage(_) => 3,
        _ => panic!("Invalid operand for CPX: {:?}", op),
      },
      Instruction::CPY(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::ZeroPage(_) => 3,
        _ => panic!("Invalid operand for CPY: {:?}", op),
      },
      Instruction::DEC(op) => match op {
        Operand::Absolute(_) => 6,
        Operand::AbsoluteX(_) => 7,
        Operand::ZeroPage(_) => 5,
        Operand::ZeroPageX(_) => 6,
        _ => panic!("Invalid operand for DEC: {:?}", op),
      },
      Instruction::DEX => 2,
      Instruction::DEY => 2,
      Instruction::EOR(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for EOR: {:?}", op),
      },
      Instruction::INC(op) => match op {
        Operand::Absolute(_) => 6,
        Operand::AbsoluteX(_) => 7,
        Operand::ZeroPage(_) => 5,
        Operand::ZeroPageX(_) => 6,
        _ => panic!("Invalid operand for INC: {:?}", op),
      },
      Instruction::INX => 2,
      Instruction::INY => 2,
      Instruction::JMP(op) => match op {
        Operand::Absolute(_) => 3,
        Operand::Indirect(_) => 5,
        _ => panic!("Invalid operand for JMP: {:?}", op),
      },
      Instruction::JSR(op) => match op {
        Operand::Absolute(_) => 6,
        _ => panic!("Invalid operand for JSR: {:?}", op),
      },
      Instruction::LDA(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for LDA: {:?}", op),
      },
      Instruction::LDX(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageY(_) => 4,
        _ => panic!("Invalid operand for LDX: {:?}", op),
      },
      Instruction::LDY(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        _ => panic!("Invalid operand for LDY: {:?}", op),
      },
      Instruction::LSR(op) => match op {
        Operand::Accumulator => 2,
        Operand::ZeroPage(_) => 5,
        Operand::ZeroPageX(_) => 6,
        Operand::Absolute(_) => 6,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for LSR: {:?}", op),
      },
      Instruction::NOP => 2,
      Instruction::ORA(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for ORA: {:?}", op),
      },
      Instruction::PHA => 3,
      Instruction::PHP => 3,
      Instruction::PLA => 4,
      Instruction::PLP => 4,
      Instruction::ROL(op) => match op {
        Operand::Accumulator => 2,
        Operand::Absolute(_) => 6,
        Operand::AbsoluteX(_) => 7,
        Operand::ZeroPage(_) => 5,
        Operand::ZeroPageX(_) => 6,
        _ => panic!("Invalid operand for ROL: {:?}", op),
      },
      Instruction::ROR(op) => match op {
        Operand::Accumulator => 2,
        Operand::Absolute(_) => 6,
        Operand::AbsoluteX(_) => 7,
        Operand::ZeroPage(_) => 5,
        Operand::ZeroPageX(_) => 6,
        _ => panic!("Invalid operand for ROR: {:?}", op),
      },
      Instruction::RTI => 6,
      Instruction::RTS => 6,
      Instruction::SBC(op) => match op {
        Operand::Immediate(_) => 2,
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for SBC: {:?}", op),
      },
      Instruction::SEC => 2,
      Instruction::SED => 2,
      Instruction::SEI => 2,
      Instruction::STA(op) => match op {
        Operand::Absolute(_) => 4,
        Operand::AbsoluteX(_) => 5,
        Operand::AbsoluteY(_) => 5,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 6,
        _ => panic!("Invalid operand for STA: {:?}", op),
      },
      Instruction::STX(op) => match op {
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageY(_) => 4,
        Operand::Absolute(_) => 4,
        _ => panic!("Invalid operand for STX: {:?}", op),
      },
      Instruction::STY(op) => match op {
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageX(_) => 4,
        Operand::Absolute(_) => 4,
        _ => panic!("Invalid operand for STY: {:?}", op),
      },
      Instruction::TAX => 2,
      Instruction::TAY => 2,
      Instruction::TSX => 2,
      Instruction::TXA => 2,
      Instruction::TXS => 2,
      Instruction::TYA => 2,
    }
  }
}

impl CPU {
  fn load_byte(&mut self, state: &Machine) -> u8 {
    let byte = state.get_cpu_mem(self.pc);
    self.pc += 1;
    byte
  }

  fn load_addr(&mut self, state: &Machine) -> u16 {
    let low = self.load_byte(state);
    let high = self.load_byte(state);

    (u16::from(high) << 8) + u16::from(low)
  }

  fn load_offset(&mut self, state: &Machine) -> i8 {
    let byte = self.load_byte(state);
    byte as i8
  }

  pub fn load_instruction(&mut self, state: &Machine) -> Instruction {
    let opcode = self.load_byte(state);

    match opcode {
      0x00 => Instruction::BRK,
      0x01 => Instruction::ORA(Operand::IndirectX(self.load_byte(state))),
      0x05 => Instruction::ORA(Operand::ZeroPage(self.load_byte(state))),
      0x06 => Instruction::ASL(Operand::ZeroPage(self.load_byte(state))),
      0x08 => Instruction::PHP,
      0x09 => Instruction::ORA(Operand::Immediate(self.load_byte(state))),
      0x0a => Instruction::ASL(Operand::Accumulator),
      0x0d => Instruction::ORA(Operand::Absolute(self.load_addr(state))),
      0x0e => Instruction::ASL(Operand::Absolute(self.load_addr(state))),
      0x10 => Instruction::BPL(Operand::Relative(self.load_offset(state))),
      0x11 => Instruction::ORA(Operand::IndirectY(self.load_byte(state))),
      0x15 => Instruction::ORA(Operand::ZeroPageX(self.load_byte(state))),
      0x16 => Instruction::ASL(Operand::ZeroPageX(self.load_byte(state))),
      0x18 => Instruction::CLC,
      0x19 => Instruction::ORA(Operand::AbsoluteY(self.load_addr(state))),
      0x1d => Instruction::ORA(Operand::AbsoluteX(self.load_addr(state))),
      0x1e => Instruction::ASL(Operand::AbsoluteX(self.load_addr(state))),
      0x20 => Instruction::JSR(Operand::Absolute(self.load_addr(state))),
      0x21 => Instruction::AND(Operand::IndirectX(self.load_byte(state))),
      0x24 => Instruction::BIT(Operand::ZeroPage(self.load_byte(state))),
      0x25 => Instruction::AND(Operand::ZeroPage(self.load_byte(state))),
      0x26 => Instruction::ROL(Operand::ZeroPage(self.load_byte(state))),
      0x28 => Instruction::PLP,
      0x29 => Instruction::AND(Operand::Immediate(self.load_byte(state))),
      0x2a => Instruction::ROL(Operand::Accumulator),
      0x2c => Instruction::BIT(Operand::Absolute(self.load_addr(state))),
      0x2d => Instruction::AND(Operand::Absolute(self.load_addr(state))),
      0x2e => Instruction::ROL(Operand::Absolute(self.load_addr(state))),
      0x30 => Instruction::BMI(Operand::Relative(self.load_offset(state))),
      0x31 => Instruction::AND(Operand::IndirectY(self.load_byte(state))),
      0x35 => Instruction::AND(Operand::ZeroPageX(self.load_byte(state))),
      0x36 => Instruction::ROL(Operand::ZeroPageX(self.load_byte(state))),
      0x38 => Instruction::SEC,
      0x39 => Instruction::AND(Operand::AbsoluteY(self.load_addr(state))),
      0x3d => Instruction::AND(Operand::AbsoluteX(self.load_addr(state))),
      0x3e => Instruction::ROL(Operand::AbsoluteX(self.load_addr(state))),
      0x40 => Instruction::RTI,
      0x41 => Instruction::EOR(Operand::IndirectX(self.load_byte(state))),
      0x45 => Instruction::EOR(Operand::ZeroPage(self.load_byte(state))),
      0x46 => Instruction::LSR(Operand::ZeroPage(self.load_byte(state))),
      0x48 => Instruction::PHA,
      0x49 => Instruction::EOR(Operand::Immediate(self.load_byte(state))),
      0x4a => Instruction::LSR(Operand::Accumulator),
      0x4c => Instruction::JMP(Operand::Absolute(self.load_addr(state))),
      0x4d => Instruction::EOR(Operand::Absolute(self.load_addr(state))),
      0x4e => Instruction::LSR(Operand::Absolute(self.load_addr(state))),
      0x50 => Instruction::BVC(Operand::Relative(self.load_offset(state))),
      0x51 => Instruction::EOR(Operand::IndirectY(self.load_byte(state))),
      0x55 => Instruction::EOR(Operand::ZeroPageX(self.load_byte(state))),
      0x56 => Instruction::LSR(Operand::ZeroPageX(self.load_byte(state))),
      0x58 => Instruction::CLI,
      0x59 => Instruction::EOR(Operand::AbsoluteY(self.load_addr(state))),
      0x5d => Instruction::EOR(Operand::AbsoluteX(self.load_addr(state))),
      0x5e => Instruction::LSR(Operand::AbsoluteX(self.load_addr(state))),
      0x60 => Instruction::RTS,
      0x61 => Instruction::ADC(Operand::IndirectX(self.load_byte(state))),
      0x65 => Instruction::ADC(Operand::ZeroPage(self.load_byte(state))),
      0x66 => Instruction::ROR(Operand::ZeroPage(self.load_byte(state))),
      0x68 => Instruction::PLA,
      0x69 => Instruction::ADC(Operand::Immediate(self.load_byte(state))),
      0x6a => Instruction::ROR(Operand::Accumulator),
      0x6c => Instruction::JMP(Operand::Indirect(self.load_addr(state))),
      0x6d => Instruction::ADC(Operand::Absolute(self.load_addr(state))),
      0x6e => Instruction::ROR(Operand::Absolute(self.load_addr(state))),
      0x70 => Instruction::BVS(Operand::Relative(self.load_offset(state))),
      0x71 => Instruction::ADC(Operand::IndirectY(self.load_byte(state))),
      0x75 => Instruction::ADC(Operand::ZeroPageX(self.load_byte(state))),
      0x76 => Instruction::ROR(Operand::ZeroPageX(self.load_byte(state))),
      0x78 => Instruction::SEI,
      0x79 => Instruction::ADC(Operand::AbsoluteY(self.load_addr(state))),
      0x7d => Instruction::ADC(Operand::AbsoluteX(self.load_addr(state))),
      0x7e => Instruction::ROR(Operand::AbsoluteX(self.load_addr(state))),
      0x9a => Instruction::TXS,
      0x81 => Instruction::STA(Operand::IndirectX(self.load_byte(state))),
      0x84 => Instruction::STY(Operand::ZeroPage(self.load_byte(state))),
      0x85 => Instruction::STA(Operand::ZeroPage(self.load_byte(state))),
      0x86 => Instruction::STX(Operand::ZeroPage(self.load_byte(state))),
      0x88 => Instruction::DEY,
      0x8a => Instruction::TXA,
      0x8c => Instruction::STY(Operand::Absolute(self.load_addr(state))),
      0x8d => Instruction::STA(Operand::Absolute(self.load_addr(state))),
      0x8e => Instruction::STX(Operand::Absolute(self.load_addr(state))),
      0x90 => Instruction::BCC(Operand::Relative(self.load_offset(state))),
      0x91 => Instruction::STA(Operand::IndirectY(self.load_byte(state))),
      0x94 => Instruction::STY(Operand::ZeroPageX(self.load_byte(state))),
      0x95 => Instruction::STA(Operand::ZeroPageX(self.load_byte(state))),
      0x96 => Instruction::STX(Operand::ZeroPageY(self.load_byte(state))),
      0x98 => Instruction::TYA,
      0x99 => Instruction::STA(Operand::AbsoluteY(self.load_addr(state))),
      0x9d => Instruction::STA(Operand::AbsoluteX(self.load_addr(state))),
      0xa0 => Instruction::LDY(Operand::Immediate(self.load_byte(state))),
      0xa1 => Instruction::LDX(Operand::IndirectX(self.load_byte(state))),
      0xa2 => Instruction::LDX(Operand::Immediate(self.load_byte(state))),
      0xa4 => Instruction::LDY(Operand::ZeroPage(self.load_byte(state))),
      0xa5 => Instruction::LDA(Operand::ZeroPage(self.load_byte(state))),
      0xa6 => Instruction::LDX(Operand::ZeroPage(self.load_byte(state))),
      0xa8 => Instruction::TAY,
      0xa9 => Instruction::LDA(Operand::Immediate(self.load_byte(state))),
      0xaa => Instruction::TAX,
      0xac => Instruction::LDY(Operand::Absolute(self.load_addr(state))),
      0xad => Instruction::LDA(Operand::Absolute(self.load_addr(state))),
      0xae => Instruction::LDX(Operand::Absolute(self.load_addr(state))),
      0xb0 => Instruction::BCS(Operand::Relative(self.load_offset(state))),
      0xb1 => Instruction::LDA(Operand::IndirectY(self.load_byte(state))),
      0xb4 => Instruction::LDY(Operand::ZeroPageX(self.load_byte(state))),
      0xb5 => Instruction::LDA(Operand::ZeroPageX(self.load_byte(state))),
      0xb6 => Instruction::LDX(Operand::ZeroPageY(self.load_byte(state))),
      0xb9 => Instruction::LDA(Operand::AbsoluteY(self.load_addr(state))),
      0xba => Instruction::TSX,
      0xbc => Instruction::LDY(Operand::AbsoluteX(self.load_addr(state))),
      0xbd => Instruction::LDA(Operand::AbsoluteX(self.load_addr(state))),
      0xbe => Instruction::LDX(Operand::AbsoluteY(self.load_addr(state))),
      0xb8 => Instruction::CLV,
      0xc0 => Instruction::CPY(Operand::Immediate(self.load_byte(state))),
      0xc1 => Instruction::CMP(Operand::IndirectX(self.load_byte(state))),
      0xc4 => Instruction::CPY(Operand::ZeroPage(self.load_byte(state))),
      0xc5 => Instruction::CMP(Operand::ZeroPage(self.load_byte(state))),
      0xc6 => Instruction::DEC(Operand::ZeroPage(self.load_byte(state))),
      0xc8 => Instruction::INY,
      0xc9 => Instruction::CMP(Operand::Immediate(self.load_byte(state))),
      0xca => Instruction::DEX,
      0xcc => Instruction::CPY(Operand::Absolute(self.load_addr(state))),
      0xcd => Instruction::CMP(Operand::Absolute(self.load_addr(state))),
      0xce => Instruction::DEC(Operand::Absolute(self.load_addr(state))),
      0xd0 => Instruction::BNE(Operand::Relative(self.load_offset(state))),
      0xd1 => Instruction::CMP(Operand::IndirectY(self.load_byte(state))),
      0xd5 => Instruction::CMP(Operand::ZeroPageX(self.load_byte(state))),
      0xd6 => Instruction::DEC(Operand::ZeroPageX(self.load_byte(state))),
      0xd8 => Instruction::CLD,
      0xd9 => Instruction::CMP(Operand::AbsoluteY(self.load_addr(state))),
      0xdd => Instruction::CMP(Operand::AbsoluteX(self.load_addr(state))),
      0xde => Instruction::DEC(Operand::AbsoluteX(self.load_addr(state))),
      0xe0 => Instruction::CPX(Operand::Immediate(self.load_byte(state))),
      0xe1 => Instruction::SBC(Operand::IndirectX(self.load_byte(state))),
      0xe4 => Instruction::CPX(Operand::ZeroPage(self.load_byte(state))),
      0xe5 => Instruction::SBC(Operand::ZeroPage(self.load_byte(state))),
      0xe6 => Instruction::INC(Operand::ZeroPage(self.load_byte(state))),
      0xe8 => Instruction::INX,
      0xe9 => Instruction::SBC(Operand::Immediate(self.load_byte(state))),
      0xea => Instruction::NOP,
      0xec => Instruction::CPX(Operand::Absolute(self.load_addr(state))),
      0xed => Instruction::SBC(Operand::Absolute(self.load_addr(state))),
      0xee => Instruction::INC(Operand::Absolute(self.load_addr(state))),
      0xf0 => Instruction::BEQ(Operand::Relative(self.load_offset(state))),
      0xf1 => Instruction::SBC(Operand::IndirectY(self.load_byte(state))),
      0xf5 => Instruction::SBC(Operand::ZeroPageX(self.load_byte(state))),
      0xf6 => Instruction::INC(Operand::ZeroPageX(self.load_byte(state))),
      0xf8 => Instruction::SED,
      0xf9 => Instruction::SBC(Operand::AbsoluteY(self.load_addr(state))),
      0xfd => Instruction::SBC(Operand::AbsoluteX(self.load_addr(state))),
      0xfe => Instruction::INC(Operand::AbsoluteX(self.load_addr(state))),

      _ => {
        panic!("Unknown opcode {:#04x}", opcode);
      }
    }
  }
}

impl Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let instruction_name: &'static str = self.into();

    match self {
      Instruction::ADC(op)
      | Instruction::AND(op)
      | Instruction::ASL(op)
      | Instruction::BCC(op)
      | Instruction::BCS(op)
      | Instruction::BEQ(op)
      | Instruction::BIT(op)
      | Instruction::BMI(op)
      | Instruction::BNE(op)
      | Instruction::BPL(op)
      | Instruction::BVC(op)
      | Instruction::BVS(op)
      | Instruction::CMP(op)
      | Instruction::CPX(op)
      | Instruction::CPY(op)
      | Instruction::DEC(op)
      | Instruction::EOR(op)
      | Instruction::INC(op)
      | Instruction::JMP(op)
      | Instruction::JSR(op)
      | Instruction::LDA(op)
      | Instruction::LDX(op)
      | Instruction::LDY(op)
      | Instruction::LSR(op)
      | Instruction::ORA(op)
      | Instruction::ROL(op)
      | Instruction::ROR(op)
      | Instruction::SBC(op)
      | Instruction::STA(op)
      | Instruction::STX(op)
      | Instruction::STY(op) => {
        f.write_fmt(format_args!("{} {}", instruction_name.to_lowercase(), op))
      }

      Instruction::BRK
      | Instruction::CLC
      | Instruction::CLD
      | Instruction::CLI
      | Instruction::CLV
      | Instruction::DEX
      | Instruction::DEY
      | Instruction::INX
      | Instruction::INY
      | Instruction::NOP
      | Instruction::PHA
      | Instruction::PHP
      | Instruction::PLA
      | Instruction::PLP
      | Instruction::RTI
      | Instruction::RTS
      | Instruction::SEC
      | Instruction::SED
      | Instruction::SEI
      | Instruction::TAX
      | Instruction::TAY
      | Instruction::TSX
      | Instruction::TXA
      | Instruction::TXS
      | Instruction::TYA => f.write_str(instruction_name.to_lowercase().as_str()),
    }
  }
}
