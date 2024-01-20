use std::fmt::{Debug, Display};

use crate::ppu::{PPUAddressLatch, PPU};

use super::{CPUBusTrait, Instruction, Operand, CPU};

#[derive(Clone, Debug)]
pub struct ExecutedInstruction {
  pub instruction: Instruction,
  pub opcode: u8,
  pub disassembled_instruction: DisassembledInstruction,
}

#[derive(Clone, Debug)]
pub struct DisassemblyMachineState {
  pub cpu: CPU,
  pub scanline: i32,
  pub cycle: i32,
  pub cycle_count: u64,
  pub vram_addr: u16,
  pub tram_addr: u16,
  pub ppu2002: u8,
  pub ppu2004: u8,
  pub ppu2007: u8,
  pub fine_x: u8,
  pub ppu_address_latch: PPUAddressLatch,
}

impl DisassemblyMachineState {
  pub fn capture(cpu: &CPU, ppu: &PPU, cpu_cycle_count: u64, cpu_bus: &dyn CPUBusTrait) -> Self {
    DisassemblyMachineState {
      cpu: cpu.clone(),
      scanline: ppu.scanline,
      cycle: ppu.cycle,
      cycle_count: cpu_cycle_count,
      vram_addr: (*cpu_bus.ppu_cpu_bus().vram_addr()).into(),
      tram_addr: (*cpu_bus.ppu_cpu_bus().tram_addr()).into(),
      ppu2002: cpu_bus.read_readonly(0x2002),
      ppu2004: cpu_bus.read_readonly(0x2004),
      ppu2007: cpu_bus.read_readonly(0x2007),
      fine_x: cpu_bus.ppu_cpu_bus().fine_x(),
      ppu_address_latch: cpu_bus.ppu_cpu_bus().address_latch(),
    }
  }
}

impl ExecutedInstruction {
  pub fn disassemble(&self, prev_state: &DisassemblyMachineState) -> String {
    format!(
      "{:04X}  {:02X} {:6}{}{:32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
      prev_state.cpu.pc,
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
      self.disassembled_instruction.to_string(),
      prev_state.cpu.a,
      prev_state.cpu.x,
      prev_state.cpu.y,
      u8::from(prev_state.cpu.p),
      prev_state.cpu.s,
      prev_state.scanline + 1,
      prev_state.cycle,
      prev_state.cycle_count
    )
  }

  pub fn disassemble_with_ppu(&self, prev_state: &DisassemblyMachineState) -> String {
    format!(
      "{:04X}  {:02X} {:6}{}{:32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} v:{:04X} t:{:04X} x:{} w:{} CYC:{:3} SL:{:<3} 2002:{:02X} 2004:{:02X} 2007:{:02X}",
      prev_state.cpu.pc,
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
      self.disassembled_instruction.to_string(),
      prev_state.cpu.a,
      prev_state.cpu.x,
      prev_state.cpu.y,
      u8::from(prev_state.cpu.p),
      prev_state.cpu.s,
      prev_state.vram_addr,
      prev_state.tram_addr,
      prev_state.fine_x,
      match prev_state.ppu_address_latch {
        PPUAddressLatch::High => 0,
        PPUAddressLatch::Low => 1
      },
      prev_state.cycle,
      prev_state.scanline,
      prev_state.ppu2002,
      prev_state.ppu2004,
      prev_state.ppu2007
    )
  }
}

#[derive(Clone, Debug)]
pub struct DisassembledInstruction {
  instruction: Instruction,
  operand: Option<DisassembledOperand>,
}

impl Display for DisassembledInstruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let instruction_name: &'static str = match &self.instruction {
      Instruction::Illegal(instruction, _op) => <&'static str>::from(*instruction.clone()),
      _ => (&self.instruction).into(),
    };

    match &self.operand {
      Some(op) => f.write_fmt(format_args!("{} {}", instruction_name, op)),
      None => f.write_str(instruction_name),
    }
  }
}

impl Instruction {
  pub fn disassemble(&self, cpu_bus: &dyn CPUBusTrait, cpu: &CPU) -> DisassembledInstruction {
    let eval = match &self {
      Instruction::JSR(_) => false,
      Instruction::JMP(op) => matches!(op, &Operand::Indirect(_)),
      _ => self.operand().is_some(),
    };

    match self.operand() {
      Some(op) => DisassembledInstruction {
        instruction: self.clone(),
        operand: Some(op.disassemble(cpu_bus, cpu, eval)),
      },
      None => DisassembledInstruction {
        instruction: self.clone(),
        operand: None,
      },
    }
  }
}

#[derive(Clone, Debug)]
pub struct DisassemblyEvalResult<AddrType: Clone + Debug> {
  addr: AddrType,
  value: u8,
}

#[derive(Clone, Debug)]
pub enum DisassembledOperand {
  Accumulator,
  Immediate {
    value: u8,
  },
  Absolute {
    addr: u16,
    result: Option<u8>,
  },
  AbsoluteX {
    base_addr: u16,
    result: Option<DisassemblyEvalResult<u16>>,
  },
  AbsoluteY {
    base_addr: u16,
    result: Option<DisassemblyEvalResult<u16>>,
  },
  ZeroPage {
    zp_addr: u8,
    result: Option<u8>,
  },
  ZeroPageX {
    zp_addr: u8,
    result: Option<DisassemblyEvalResult<u8>>,
  },
  ZeroPageY {
    zp_addr: u8,
    result: Option<DisassemblyEvalResult<u8>>,
  },
  Indirect {
    addr: u16,
    result_addr: Option<u16>,
  },
  IndirectX {
    zp_addr: u8,
    result: Option<(u8, DisassemblyEvalResult<u16>)>,
  },
  IndirectY {
    zp_addr: u8,
    result: Option<(u16, DisassemblyEvalResult<u16>)>,
  },
  Relative {
    offset: i8,
    pc: u16,
  },
}

impl Display for DisassembledOperand {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let operand_formatted = match self {
      Self::Accumulator => "A".to_owned(),
      Self::Immediate { value } => format!("#${:02X}", value),
      Self::Absolute { addr, .. } => format!("${:04X}", addr),
      Self::AbsoluteX { base_addr, .. } => format!("${:04X},X", base_addr),
      Self::AbsoluteY { base_addr, .. } => format!("${:04X},Y", base_addr),
      Self::ZeroPage { zp_addr, .. } => format!("${:02X}", zp_addr),
      Self::ZeroPageX { zp_addr, .. } => format!("${:02X},X", zp_addr),
      Self::ZeroPageY { zp_addr, .. } => format!("${:02X},Y", zp_addr),
      Self::Indirect { addr, .. } => format!("(${:04X})", addr),
      Self::IndirectX { zp_addr, .. } => format!("(${:02X},X)", zp_addr),
      Self::IndirectY { zp_addr, .. } => format!("(${:02X}),Y", zp_addr),
      Self::Relative { offset, pc } => {
        format!("${:04X}", *pc as i32 + *offset as i32)
      }
    };

    match self {
      Self::AbsoluteX {
        result: Some(result),
        ..
      }
      | Self::AbsoluteY {
        result: Some(result),
        ..
      } => f.write_fmt(format_args!(
        "{} @ {:04X} = {:02X}",
        operand_formatted, result.addr, result.value
      )),
      Self::ZeroPageX {
        result: Some(result),
        ..
      }
      | Self::ZeroPageY {
        result: Some(result),
        ..
      } => f.write_fmt(format_args!(
        "{} @ {:02X} = {:02X}",
        operand_formatted, result.addr, result.value
      )),
      Self::Indirect {
        result_addr: Some(result_addr),
        ..
      } => f.write_fmt(format_args!("{} = {:04X}", operand_formatted, result_addr)),
      Self::IndirectX {
        zp_addr,
        result: Some((x, result)),
      } => f.write_fmt(format_args!(
        "{} @ {:02X} = {:04X} = {:02X}",
        operand_formatted,
        x.wrapping_add(*zp_addr),
        result.addr,
        result.value
      )),
      Self::IndirectY {
        result: Some((intermediate_addr, result)),
        ..
      } => f.write_fmt(format_args!(
        "{} = {:04X} @ {:04X} = {:02X}",
        operand_formatted, intermediate_addr, result.addr, result.value
      )),
      Self::Absolute {
        result: Some(value),
        ..
      }
      | Self::ZeroPage {
        result: Some(value),
        ..
      } => f.write_fmt(format_args!("{} = {:02X}", operand_formatted, value)),
      _ => f.write_str(&operand_formatted),
    }
  }
}

impl Operand {
  fn disassemble(&self, cpu_bus: &dyn CPUBusTrait, cpu: &CPU, eval: bool) -> DisassembledOperand {
    match self {
      Operand::Accumulator => DisassembledOperand::Accumulator,
      Operand::Immediate(value) => DisassembledOperand::Immediate { value: *value },
      Operand::Absolute(addr) => DisassembledOperand::Absolute {
        addr: *addr,
        result: if eval {
          Some(self.eval_readonly(cpu, cpu_bus).0)
        } else {
          None
        },
      },
      Operand::AbsoluteX(base_addr) => DisassembledOperand::AbsoluteX {
        base_addr: *base_addr,
        result: self.get_eval_result(cpu_bus, cpu, eval, || {
          self.get_addr_readonly(cpu, cpu_bus).0
        }),
      },
      Operand::AbsoluteY(base_addr) => DisassembledOperand::AbsoluteY {
        base_addr: *base_addr,
        result: self.get_eval_result(cpu_bus, cpu, eval, || {
          self.get_addr_readonly(cpu, cpu_bus).0
        }),
      },
      Operand::ZeroPage(zp_addr) => DisassembledOperand::ZeroPage {
        zp_addr: *zp_addr,
        result: if eval {
          Some(self.eval_readonly(cpu, cpu_bus).0)
        } else {
          None
        },
      },
      Operand::ZeroPageX(zp_addr) => DisassembledOperand::ZeroPageX {
        zp_addr: *zp_addr,
        result: self.get_eval_result(cpu_bus, cpu, eval, || {
          self.get_addr_readonly(cpu, cpu_bus).0 as u8
        }),
      },
      Operand::ZeroPageY(zp_addr) => DisassembledOperand::ZeroPageY {
        zp_addr: *zp_addr,
        result: self.get_eval_result(cpu_bus, cpu, eval, || {
          self.get_addr_readonly(cpu, cpu_bus).0 as u8
        }),
      },
      Operand::Indirect(addr) => DisassembledOperand::Indirect {
        addr: *addr,
        result_addr: if eval {
          Some(self.get_addr_readonly(cpu, cpu_bus).0)
        } else {
          None
        },
      },
      Operand::IndirectX(zp_addr) => DisassembledOperand::IndirectX {
        zp_addr: *zp_addr,
        result: self
          .get_eval_result(cpu_bus, cpu, eval, || {
            self.get_addr_readonly(cpu, cpu_bus).0
          })
          .map(|result| (cpu.x, result)),
      },
      Operand::IndirectY(zp_addr) => DisassembledOperand::IndirectY {
        zp_addr: *zp_addr,
        result: self
          .get_eval_result(cpu_bus, cpu, eval, || {
            self.get_addr_readonly(cpu, cpu_bus).0
          })
          .map(|result| {
            let low = cpu_bus.read_readonly(u16::from(*zp_addr));
            let high = cpu_bus.read_readonly(u16::from(zp_addr.wrapping_add(1)));
            let intermediate_addr = (u16::from(high) << 8) + u16::from(low);
            (intermediate_addr, result)
          }),
      },
      Operand::Relative(offset) => DisassembledOperand::Relative {
        offset: *offset,
        pc: cpu.pc,
      },
    }
  }

  fn get_eval_result<AddrType: Clone + Debug, F: FnOnce() -> AddrType>(
    &self,
    cpu_bus: &dyn CPUBusTrait,
    cpu: &CPU,
    eval: bool,
    f: F,
  ) -> Option<DisassemblyEvalResult<AddrType>> {
    if eval {
      Some(DisassemblyEvalResult {
        addr: f(),
        value: self.eval_readonly(cpu, cpu_bus).0,
      })
    } else {
      None
    }
  }
}
