use strum::IntoStaticStr;

use crate::{bus::Bus, nes::NES};

use super::{Operand, CPU};

#[derive(Debug, IntoStaticStr, Clone)]
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
  Illegal(Box<Instruction>, Option<Operand>),

  // Unofficial instructions
  DCP(Operand),
  ISB(Operand),
  LAX(Operand),
  RLA(Operand),
  RRA(Operand),
  SAX(Operand), // my favorite metroid villain
  SLO(Operand),
  SRE(Operand),
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

      Instruction::Illegal(instruction, op) => match **instruction {
        Instruction::NOP => match op {
          Some(Operand::Absolute(_)) => 4,
          Some(Operand::AbsoluteX(_)) => 4,
          Some(Operand::ZeroPage(_)) => 3,
          Some(Operand::ZeroPageX(_)) => 4,
          _ => instruction.base_cycles(),
        },
        _ => instruction.base_cycles(),
      },

      // Unofficial instructions
      Self::DCP(op) => match op {
        Operand::IndirectX(_) => 8,
        Operand::ZeroPage(_) => 5,
        Operand::Absolute(_) => 6,
        Operand::IndirectY(_) => 8,
        Operand::ZeroPageX(_) => 6,
        Operand::AbsoluteY(_) => 7,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for DCP: {:?}", op),
      },
      Self::ISB(op) => match op {
        Operand::IndirectX(_) => 8,
        Operand::ZeroPage(_) => 5,
        Operand::Absolute(_) => 6,
        Operand::IndirectY(_) => 8,
        Operand::ZeroPageX(_) => 6,
        Operand::AbsoluteY(_) => 7,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for ISB: {:?}", op),
      },
      Self::LAX(op) => match op {
        Operand::Absolute(_) => 4,
        Operand::AbsoluteY(_) => 4,
        Operand::ZeroPage(_) => 3,
        Operand::ZeroPageY(_) => 4,
        Operand::IndirectX(_) => 6,
        Operand::IndirectY(_) => 5,
        _ => panic!("Invalid operand for LAX: {:?}", op),
      },
      Self::RLA(op) => match op {
        Operand::IndirectX(_) => 8,
        Operand::ZeroPage(_) => 5,
        Operand::Absolute(_) => 6,
        Operand::IndirectY(_) => 8,
        Operand::ZeroPageX(_) => 6,
        Operand::AbsoluteY(_) => 7,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for RLA: {:?}", op),
      },
      Self::RRA(op) => match op {
        Operand::IndirectX(_) => 8,
        Operand::ZeroPage(_) => 5,
        Operand::Absolute(_) => 6,
        Operand::IndirectY(_) => 8,
        Operand::ZeroPageX(_) => 6,
        Operand::AbsoluteY(_) => 7,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for RRA: {:?}", op),
      },
      Self::SAX(op) => match op {
        Operand::IndirectX(_) => 6,
        Operand::ZeroPage(_) => 3,
        Operand::Absolute(_) => 4,
        Operand::ZeroPageY(_) => 4,
        _ => panic!("Invalid operand for SAX: {:?}", op),
      },
      Self::SLO(op) => match op {
        Operand::IndirectX(_) => 8,
        Operand::ZeroPage(_) => 5,
        Operand::Absolute(_) => 6,
        Operand::IndirectY(_) => 8,
        Operand::ZeroPageX(_) => 6,
        Operand::AbsoluteY(_) => 7,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for SLO: {:?}", op),
      },
      Self::SRE(op) => match op {
        Operand::IndirectX(_) => 8,
        Operand::ZeroPage(_) => 5,
        Operand::Absolute(_) => 6,
        Operand::IndirectY(_) => 8,
        Operand::ZeroPageX(_) => 6,
        Operand::AbsoluteY(_) => 7,
        Operand::AbsoluteX(_) => 7,
        _ => panic!("Invalid operand for SRE: {:?}", op),
      },
    }
  }

  pub fn operand(&self) -> Option<Operand> {
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
      | Instruction::STY(op)

      // Unofficial instructions
      | Instruction::DCP(op)
      | Instruction::ISB(op)
      | Instruction::LAX(op)
      | Instruction::RLA(op)
      | Instruction::RRA(op)
      | Instruction::SAX(op)
      | Instruction::SLO(op)
      | Instruction::SRE(op) => Some(op.clone()),

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
      | Instruction::TYA => None,

      Instruction::Illegal(_instruction, op) => op.to_owned(),
    }
  }

  fn load_byte(nes: &mut NES) -> u8 {
    let pc = nes.cpu.pc;
    let byte = nes.cpu_bus_mut().read(pc);
    CPU::set_pc(&Operand::Absolute(nes.cpu.pc.wrapping_add(1)), nes);
    byte
  }

  fn load_addr(nes: &mut NES) -> u16 {
    let low = Instruction::load_byte(nes);
    let high = Instruction::load_byte(nes);

    (u16::from(high) << 8) + u16::from(low)
  }

  fn load_offset(nes: &mut NES) -> i8 {
    let byte = Instruction::load_byte(nes);
    byte as i8
  }

  pub fn load_instruction(nes: &mut NES) -> (Instruction, u8) {
    let opcode = Instruction::load_byte(nes);

    let instruction = match opcode {
      0x00 => Instruction::BRK,
      0x01 => Instruction::ORA(Operand::IndirectX(Instruction::load_byte(nes))),
      0x03 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }
      0x04 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPage(Instruction::load_byte(nes))),
      ),
      0x05 => Instruction::ORA(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x06 => Instruction::ASL(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x07 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }
      0x08 => Instruction::PHP,
      0x09 => Instruction::ORA(Operand::Immediate(Instruction::load_byte(nes))),
      0x0a => Instruction::ASL(Operand::Accumulator),
      0x0c => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::Absolute(Instruction::load_addr(nes))),
      ),
      0x0d => Instruction::ORA(Operand::Absolute(Instruction::load_addr(nes))),
      0x0e => Instruction::ASL(Operand::Absolute(Instruction::load_addr(nes))),
      0x0f => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }

      0x10 => Instruction::BPL(Operand::Relative(Instruction::load_offset(nes))),
      0x11 => Instruction::ORA(Operand::IndirectY(Instruction::load_byte(nes))),
      0x13 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }
      0x14 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPageX(Instruction::load_byte(nes))),
      ),
      0x15 => Instruction::ORA(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x16 => Instruction::ASL(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x17 => {
        let op = Operand::ZeroPageX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }
      0x18 => Instruction::CLC,
      0x19 => Instruction::ORA(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0x1a => Instruction::Illegal(Box::new(Instruction::NOP), None),
      0x1b => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }
      0x1c => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::AbsoluteX(Instruction::load_addr(nes))),
      ),
      0x1d => Instruction::ORA(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x1e => Instruction::ASL(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x1f => {
        let op = Operand::AbsoluteX(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SLO(op.clone())), Some(op))
      }

      0x20 => Instruction::JSR(Operand::Absolute(Instruction::load_addr(nes))),
      0x21 => Instruction::AND(Operand::IndirectX(Instruction::load_byte(nes))),
      0x23 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }
      0x24 => Instruction::BIT(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x25 => Instruction::AND(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x26 => Instruction::ROL(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x27 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }
      0x28 => Instruction::PLP,
      0x29 => Instruction::AND(Operand::Immediate(Instruction::load_byte(nes))),
      0x2a => Instruction::ROL(Operand::Accumulator),
      0x2c => Instruction::BIT(Operand::Absolute(Instruction::load_addr(nes))),
      0x2d => Instruction::AND(Operand::Absolute(Instruction::load_addr(nes))),
      0x2e => Instruction::ROL(Operand::Absolute(Instruction::load_addr(nes))),
      0x2f => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }

      0x30 => Instruction::BMI(Operand::Relative(Instruction::load_offset(nes))),
      0x31 => Instruction::AND(Operand::IndirectY(Instruction::load_byte(nes))),
      0x33 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }
      0x34 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPageX(Instruction::load_byte(nes))),
      ),
      0x35 => Instruction::AND(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x36 => Instruction::ROL(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x37 => {
        let op = Operand::ZeroPageX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }
      0x38 => Instruction::SEC,
      0x39 => Instruction::AND(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0x3a => Instruction::Illegal(Box::new(Instruction::NOP), None),
      0x3b => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }
      0x3c => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::AbsoluteX(Instruction::load_addr(nes))),
      ),
      0x3d => Instruction::AND(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x3e => Instruction::ROL(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x3f => {
        let op = Operand::AbsoluteX(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::RLA(op.clone())), Some(op))
      }

      0x40 => Instruction::RTI,
      0x41 => Instruction::EOR(Operand::IndirectX(Instruction::load_byte(nes))),
      0x43 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }
      0x44 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPage(Instruction::load_byte(nes))),
      ),
      0x45 => Instruction::EOR(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x46 => Instruction::LSR(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x47 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }
      0x48 => Instruction::PHA,
      0x49 => Instruction::EOR(Operand::Immediate(Instruction::load_byte(nes))),
      0x4a => Instruction::LSR(Operand::Accumulator),
      0x4c => Instruction::JMP(Operand::Absolute(Instruction::load_addr(nes))),
      0x4d => Instruction::EOR(Operand::Absolute(Instruction::load_addr(nes))),
      0x4e => Instruction::LSR(Operand::Absolute(Instruction::load_addr(nes))),
      0x4f => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }

      0x50 => Instruction::BVC(Operand::Relative(Instruction::load_offset(nes))),
      0x51 => Instruction::EOR(Operand::IndirectY(Instruction::load_byte(nes))),
      0x53 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }
      0x54 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPageX(Instruction::load_byte(nes))),
      ),
      0x55 => Instruction::EOR(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x56 => Instruction::LSR(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x57 => {
        let op = Operand::ZeroPageX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }
      0x58 => Instruction::CLI,
      0x59 => Instruction::EOR(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0x5a => Instruction::Illegal(Box::new(Instruction::NOP), None),
      0x5b => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }
      0x5c => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::AbsoluteX(Instruction::load_addr(nes))),
      ),
      0x5d => Instruction::EOR(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x5e => Instruction::LSR(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x5f => {
        let op = Operand::AbsoluteX(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SRE(op.clone())), Some(op))
      }

      0x60 => Instruction::RTS,
      0x61 => Instruction::ADC(Operand::IndirectX(Instruction::load_byte(nes))),
      0x63 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }
      0x64 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPage(Instruction::load_byte(nes))),
      ),
      0x65 => Instruction::ADC(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x66 => Instruction::ROR(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x67 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }
      0x68 => Instruction::PLA,
      0x69 => Instruction::ADC(Operand::Immediate(Instruction::load_byte(nes))),
      0x6a => Instruction::ROR(Operand::Accumulator),
      0x6c => Instruction::JMP(Operand::Indirect(Instruction::load_addr(nes))),
      0x6d => Instruction::ADC(Operand::Absolute(Instruction::load_addr(nes))),
      0x6e => Instruction::ROR(Operand::Absolute(Instruction::load_addr(nes))),
      0x6f => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }

      0x70 => Instruction::BVS(Operand::Relative(Instruction::load_offset(nes))),
      0x71 => Instruction::ADC(Operand::IndirectY(Instruction::load_byte(nes))),
      0x73 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }
      0x74 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPageX(Instruction::load_byte(nes))),
      ),
      0x75 => Instruction::ADC(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x76 => Instruction::ROR(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x77 => {
        let op = Operand::ZeroPageX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }
      0x78 => Instruction::SEI,
      0x79 => Instruction::ADC(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0x7a => Instruction::Illegal(Box::new(Instruction::NOP), None),
      0x7b => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }
      0x7c => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::AbsoluteX(Instruction::load_addr(nes))),
      ),
      0x7d => Instruction::ADC(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x7e => Instruction::ROR(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0x7f => {
        let op = Operand::AbsoluteX(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::RRA(op.clone())), Some(op))
      }

      0x80 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::Immediate(Instruction::load_byte(nes))),
      ),
      0x81 => Instruction::STA(Operand::IndirectX(Instruction::load_byte(nes))),
      0x83 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SAX(op.clone())), Some(op))
      }
      0x84 => Instruction::STY(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x85 => Instruction::STA(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x86 => Instruction::STX(Operand::ZeroPage(Instruction::load_byte(nes))),
      0x87 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SAX(op.clone())), Some(op))
      }
      0x88 => Instruction::DEY,
      0x8a => Instruction::TXA,
      0x8c => Instruction::STY(Operand::Absolute(Instruction::load_addr(nes))),
      0x8d => Instruction::STA(Operand::Absolute(Instruction::load_addr(nes))),
      0x8e => Instruction::STX(Operand::Absolute(Instruction::load_addr(nes))),
      0x8f => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::SAX(op.clone())), Some(op))
      }

      0x90 => Instruction::BCC(Operand::Relative(Instruction::load_offset(nes))),
      0x91 => Instruction::STA(Operand::IndirectY(Instruction::load_byte(nes))),
      0x94 => Instruction::STY(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x95 => Instruction::STA(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0x96 => Instruction::STX(Operand::ZeroPageY(Instruction::load_byte(nes))),
      0x97 => {
        let op = Operand::ZeroPageY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SAX(op.clone())), Some(op))
      }
      0x98 => Instruction::TYA,
      0x99 => Instruction::STA(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0x9a => Instruction::TXS,
      0x9d => Instruction::STA(Operand::AbsoluteX(Instruction::load_addr(nes))),

      0xa0 => Instruction::LDY(Operand::Immediate(Instruction::load_byte(nes))),
      0xa1 => Instruction::LDA(Operand::IndirectX(Instruction::load_byte(nes))),
      0xa2 => Instruction::LDX(Operand::Immediate(Instruction::load_byte(nes))),
      0xa3 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::LAX(op.clone())), Some(op))
      }
      0xa4 => Instruction::LDY(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xa5 => Instruction::LDA(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xa6 => Instruction::LDX(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xa7 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::LAX(op.clone())), Some(op))
      }
      0xa8 => Instruction::TAY,
      0xa9 => Instruction::LDA(Operand::Immediate(Instruction::load_byte(nes))),
      0xaa => Instruction::TAX,
      0xac => Instruction::LDY(Operand::Absolute(Instruction::load_addr(nes))),
      0xad => Instruction::LDA(Operand::Absolute(Instruction::load_addr(nes))),
      0xae => Instruction::LDX(Operand::Absolute(Instruction::load_addr(nes))),
      0xaf => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::LAX(op.clone())), Some(op))
      }

      0xb0 => Instruction::BCS(Operand::Relative(Instruction::load_offset(nes))),
      0xb1 => Instruction::LDA(Operand::IndirectY(Instruction::load_byte(nes))),
      0xb3 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::LAX(op.clone())), Some(op))
      }
      0xb4 => Instruction::LDY(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0xb5 => Instruction::LDA(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0xb6 => Instruction::LDX(Operand::ZeroPageY(Instruction::load_byte(nes))),
      0xb7 => {
        let op = Operand::ZeroPageY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::LAX(op.clone())), Some(op))
      }
      0xb9 => Instruction::LDA(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0xba => Instruction::TSX,
      0xbc => Instruction::LDY(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0xbd => Instruction::LDA(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0xbe => Instruction::LDX(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0xb8 => Instruction::CLV,
      0xbf => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::LAX(op.clone())), Some(op))
      }

      0xc0 => Instruction::CPY(Operand::Immediate(Instruction::load_byte(nes))),
      0xc1 => Instruction::CMP(Operand::IndirectX(Instruction::load_byte(nes))),
      0xc3 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }
      0xc4 => Instruction::CPY(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xc5 => Instruction::CMP(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xc6 => Instruction::DEC(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xc7 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }
      0xc8 => Instruction::INY,
      0xc9 => Instruction::CMP(Operand::Immediate(Instruction::load_byte(nes))),
      0xca => Instruction::DEX,
      0xcc => Instruction::CPY(Operand::Absolute(Instruction::load_addr(nes))),
      0xcd => Instruction::CMP(Operand::Absolute(Instruction::load_addr(nes))),
      0xce => Instruction::DEC(Operand::Absolute(Instruction::load_addr(nes))),
      0xcf => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }

      0xd0 => Instruction::BNE(Operand::Relative(Instruction::load_offset(nes))),
      0xd1 => Instruction::CMP(Operand::IndirectY(Instruction::load_byte(nes))),
      0xd3 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }
      0xd4 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPageX(Instruction::load_byte(nes))),
      ),
      0xd5 => Instruction::CMP(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0xd6 => Instruction::DEC(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0xd7 => {
        let op = Operand::ZeroPageX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }
      0xd8 => Instruction::CLD,
      0xd9 => Instruction::CMP(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0xda => Instruction::Illegal(Box::new(Instruction::NOP), None),
      0xdb => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }
      0xdc => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::AbsoluteX(Instruction::load_addr(nes))),
      ),
      0xdd => Instruction::CMP(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0xde => Instruction::DEC(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0xdf => {
        let op = Operand::AbsoluteX(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::DCP(op.clone())), Some(op))
      }

      0xe0 => Instruction::CPX(Operand::Immediate(Instruction::load_byte(nes))),
      0xe1 => Instruction::SBC(Operand::IndirectX(Instruction::load_byte(nes))),
      0xe3 => {
        let op = Operand::IndirectX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }
      0xe4 => Instruction::CPX(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xe5 => Instruction::SBC(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xe6 => Instruction::INC(Operand::ZeroPage(Instruction::load_byte(nes))),
      0xe7 => {
        let op = Operand::ZeroPage(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }
      0xe8 => Instruction::INX,
      0xe9 => Instruction::SBC(Operand::Immediate(Instruction::load_byte(nes))),
      0xea => Instruction::NOP,
      0xeb => {
        let op = Operand::Immediate(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::SBC(op.clone())), Some(op))
      }
      0xec => Instruction::CPX(Operand::Absolute(Instruction::load_addr(nes))),
      0xed => Instruction::SBC(Operand::Absolute(Instruction::load_addr(nes))),
      0xee => Instruction::INC(Operand::Absolute(Instruction::load_addr(nes))),
      0xef => {
        let op = Operand::Absolute(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }

      0xf0 => Instruction::BEQ(Operand::Relative(Instruction::load_offset(nes))),
      0xf1 => Instruction::SBC(Operand::IndirectY(Instruction::load_byte(nes))),
      0xf3 => {
        let op = Operand::IndirectY(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }
      0xf4 => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::ZeroPageX(Instruction::load_byte(nes))),
      ),
      0xf5 => Instruction::SBC(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0xf6 => Instruction::INC(Operand::ZeroPageX(Instruction::load_byte(nes))),
      0xf7 => {
        let op = Operand::ZeroPageX(Instruction::load_byte(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }
      0xf8 => Instruction::SED,
      0xf9 => Instruction::SBC(Operand::AbsoluteY(Instruction::load_addr(nes))),
      0xfb => {
        let op = Operand::AbsoluteY(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }
      0xfc => Instruction::Illegal(
        Box::new(Instruction::NOP),
        Some(Operand::AbsoluteX(Instruction::load_addr(nes))),
      ),
      0xfa => Instruction::Illegal(Box::new(Instruction::NOP), None),
      0xfd => Instruction::SBC(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0xfe => Instruction::INC(Operand::AbsoluteX(Instruction::load_addr(nes))),
      0xff => {
        let op = Operand::AbsoluteX(Instruction::load_addr(nes));
        Instruction::Illegal(Box::new(Instruction::ISB(op.clone())), Some(op))
      }

      _ => {
        panic!("Unknown opcode {:#04x}", opcode);
      }
    };

    (instruction, opcode)
  }
}
