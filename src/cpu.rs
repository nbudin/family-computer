use crate::memory::Memory;

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
  TXA,
  TXS,
  TYA,
}

#[derive(Debug)]
pub struct CPU {
  pub pc: usize,
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

  fn load_prg_byte(&mut self, prg_rom: &Vec<u8>) -> u8 {
    let byte = prg_rom.get(self.pc).expect("PC is out of bounds");
    self.pc += 1;
    *byte
  }

  fn load_prg_addr(&mut self, prg_rom: &Vec<u8>) -> u16 {
    let low = self.load_prg_byte(prg_rom);
    let high = self.load_prg_byte(prg_rom);

    (u16::from(high) << 8) + u16::from(low)
  }

  fn load_prg_offset(&mut self, prg_rom: &Vec<u8>) -> i8 {
    let byte = self.load_prg_byte(prg_rom);
    byte as i8
  }

  fn load_instruction(&mut self, prg_rom: &Vec<u8>) -> Instruction {
    let opcode = self.load_prg_byte(prg_rom);

    match opcode {
      0x00 => Instruction::BRK,
      0x01 => Instruction::ORA(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0x05 => Instruction::ORA(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x06 => Instruction::ASL(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x08 => Instruction::PHP,
      0x09 => Instruction::ORA(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0x0a => Instruction::ASL(Operand::Accumulator),
      0x0d => Instruction::ORA(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x0e => Instruction::ASL(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x10 => Instruction::BPL(Operand::Relative(self.load_prg_offset(prg_rom))),
      0x11 => Instruction::ORA(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0x15 => Instruction::ORA(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x16 => Instruction::ASL(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x18 => Instruction::CLC,
      0x19 => Instruction::ORA(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0x1d => Instruction::ORA(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x1e => Instruction::ASL(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x20 => Instruction::JSR(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x21 => Instruction::AND(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0x24 => Instruction::BIT(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x25 => Instruction::AND(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x26 => Instruction::ROL(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x28 => Instruction::PLP,
      0x29 => Instruction::AND(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0x2a => Instruction::ROL(Operand::Accumulator),
      0x2c => Instruction::BIT(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x2d => Instruction::AND(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x2e => Instruction::ROL(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x30 => Instruction::BMI(Operand::Relative(self.load_prg_offset(prg_rom))),
      0x31 => Instruction::AND(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0x35 => Instruction::AND(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x36 => Instruction::ROL(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x38 => Instruction::SEC,
      0x39 => Instruction::AND(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0x3d => Instruction::AND(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x3e => Instruction::ROL(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x40 => Instruction::RTI,
      0x41 => Instruction::EOR(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0x45 => Instruction::EOR(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x46 => Instruction::LSR(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x48 => Instruction::PHA,
      0x49 => Instruction::EOR(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0x4a => Instruction::LSR(Operand::Accumulator),
      0x4c => Instruction::JMP(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x4d => Instruction::EOR(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x4e => Instruction::LSR(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x50 => Instruction::BVC(Operand::Relative(self.load_prg_offset(prg_rom))),
      0x51 => Instruction::EOR(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0x55 => Instruction::EOR(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x56 => Instruction::LSR(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x58 => Instruction::CLI,
      0x59 => Instruction::EOR(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0x5d => Instruction::EOR(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x5e => Instruction::LSR(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x60 => Instruction::RTS,
      0x61 => Instruction::ADC(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0x65 => Instruction::ADC(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x66 => Instruction::ROR(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x68 => Instruction::PLA,
      0x69 => Instruction::ADC(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0x6a => Instruction::ROR(Operand::Accumulator),
      0x6c => Instruction::JMP(Operand::Indirect(self.load_prg_addr(prg_rom))),
      0x6d => Instruction::ADC(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x6e => Instruction::ROR(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x70 => Instruction::BVS(Operand::Relative(self.load_prg_offset(prg_rom))),
      0x71 => Instruction::ADC(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0x75 => Instruction::ADC(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x76 => Instruction::ROR(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x78 => Instruction::SEI,
      0x79 => Instruction::ADC(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0x7d => Instruction::ADC(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x7e => Instruction::ROR(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0x9a => Instruction::TXS,
      0x81 => Instruction::STA(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0x84 => Instruction::STY(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x85 => Instruction::STA(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x86 => Instruction::STX(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0x88 => Instruction::DEY,
      0x8a => Instruction::TXA,
      0x8c => Instruction::STY(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x8d => Instruction::STA(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x8e => Instruction::STX(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0x90 => Instruction::BCC(Operand::Relative(self.load_prg_offset(prg_rom))),
      0x91 => Instruction::STA(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0x94 => Instruction::STY(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x95 => Instruction::STA(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0x96 => Instruction::STX(Operand::ZeroPageY(self.load_prg_byte(prg_rom))),
      0x98 => Instruction::TYA,
      0x99 => Instruction::STA(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0x9d => Instruction::STA(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0xa0 => Instruction::LDY(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xa1 => Instruction::LDX(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0xa2 => Instruction::LDX(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xa4 => Instruction::LDY(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xa5 => Instruction::LDA(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xa6 => Instruction::LDX(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xa8 => Instruction::TAY,
      0xa9 => Instruction::LDA(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xaa => Instruction::TAX,
      0xac => Instruction::LDY(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xad => Instruction::LDA(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xae => Instruction::LDX(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xb0 => Instruction::BCS(Operand::Relative(self.load_prg_offset(prg_rom))),
      0xb1 => Instruction::LDX(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0xb4 => Instruction::LDY(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0xb5 => Instruction::LDA(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0xb6 => Instruction::LDX(Operand::ZeroPageY(self.load_prg_byte(prg_rom))),
      0xb9 => Instruction::LDA(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0xbc => Instruction::LDY(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0xbd => Instruction::LDA(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0xbe => Instruction::LDX(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0xb8 => Instruction::CLV,
      0xc0 => Instruction::CPY(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xc1 => Instruction::CMP(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0xc4 => Instruction::CPY(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xc5 => Instruction::CMP(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xc6 => Instruction::DEC(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xc8 => Instruction::INY,
      0xc9 => Instruction::CMP(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xca => Instruction::DEX,
      0xcc => Instruction::CPY(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xcd => Instruction::CMP(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xce => Instruction::DEC(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xd0 => Instruction::BNE(Operand::Relative(self.load_prg_offset(prg_rom))),
      0xd1 => Instruction::CMP(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0xd5 => Instruction::CMP(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0xd6 => Instruction::DEC(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0xd8 => Instruction::CLD,
      0xd9 => Instruction::CMP(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0xdd => Instruction::CMP(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0xde => Instruction::DEC(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0xe0 => Instruction::CPX(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xe1 => Instruction::SBC(Operand::IndirectX(self.load_prg_byte(prg_rom))),
      0xe4 => Instruction::CPX(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xe5 => Instruction::SBC(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xe6 => Instruction::INC(Operand::ZeroPage(self.load_prg_byte(prg_rom))),
      0xe8 => Instruction::INX,
      0xe9 => Instruction::SBC(Operand::Immediate(self.load_prg_byte(prg_rom))),
      0xea => Instruction::NOP,
      0xec => Instruction::CPX(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xed => Instruction::SBC(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xee => Instruction::INC(Operand::Absolute(self.load_prg_addr(prg_rom))),
      0xf0 => Instruction::BEQ(Operand::Relative(self.load_prg_offset(prg_rom))),
      0xf1 => Instruction::SBC(Operand::IndirectY(self.load_prg_byte(prg_rom))),
      0xf5 => Instruction::SBC(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0xf6 => Instruction::INC(Operand::ZeroPageX(self.load_prg_byte(prg_rom))),
      0xf8 => Instruction::SED,
      0xf9 => Instruction::SBC(Operand::AbsoluteY(self.load_prg_addr(prg_rom))),
      0xfd => Instruction::SBC(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),
      0xfe => Instruction::INC(Operand::AbsoluteX(self.load_prg_addr(prg_rom))),

      _ => {
        panic!("Unknown opcode {:#04x}", opcode);
      }
    }
  }

  pub fn set_pc(&mut self, addr: &Operand) {
    match addr {
      Operand::Relative(offset) => {
        (self.pc, _) = self.pc.overflowing_add_signed(isize::from(*offset));
      }
      _ => {
        panic!("Unknown addressing mode: {:?}", addr);
      }
    }
  }

  pub fn step(&mut self, prg_rom: &Vec<u8>, memory: &mut Memory) {
    let instruction = self.load_instruction(&prg_rom);
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

      Instruction::LDA(addr) => {
        self.a = memory.get(&addr);
        self.zero_flag = self.a == 0;
        self.negative_flag = (self.a & 0b10000000) > 0;
      }

      Instruction::LDX(addr) => {
        self.x = memory.get(&addr);
        self.zero_flag = self.x == 0;
        self.negative_flag = (self.x & 0b10000000) > 0;
      }

      Instruction::LDY(addr) => {
        self.y = memory.get(&addr);
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
        memory.set(&addr, self.a);
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
