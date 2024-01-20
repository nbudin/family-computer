use std::{
  any::Any,
  fmt::Debug,
  io::Write,
  sync::{Arc, RwLock},
};

use crate::{
  apu::APUSynth,
  audio::stream_setup::StreamSpawner,
  cartridge::Cartridge,
  cpu::{DisassemblyMachineState, ExecutedInstruction, CPU},
  ppu::{Pixbuf, PPU},
};

use super::INESRom;

pub trait DisassemblyWriter: Write + Debug + Any {
  fn as_any(&self) -> &dyn Any
  where
    Self: Sized,
  {
    self
  }

  fn into_any(self) -> Box<dyn Any>
  where
    Self: Sized,
  {
    Box::new(self)
  }
}

impl<T: Write + Debug + Any> DisassemblyWriter for T {}

pub struct NESState {
  pub cartridge: Cartridge,
  pub cpu: CPU,
  pub ppu: PPU,
  pub cpu_cycle_count: u64,
  pub ppu_cycle_count: u64,
}

impl NESState {
  pub fn new(cartridge: Cartridge) -> Self {
    Self {
      cartridge,
      cpu: CPU::new(),
      ppu: PPU::new(),
      cpu_cycle_count: 0,
      ppu_cycle_count: 0,
    }
  }
}

#[allow(clippy::upper_case_acronyms)]
pub struct NES {
  pub state: NESState,
  pub apu_sender: <APUSynth as StreamSpawner>::OutputType,
  pub last_executed_instruction: Option<ExecutedInstruction>,
  pub last_disassembly_machine_state: Option<DisassemblyMachineState>,
  pub disassembly_writer: Option<Arc<RwLock<dyn DisassemblyWriter + Send + Sync>>>,
}

impl NES {
  pub fn from_rom(rom: INESRom, apu_sender: <APUSynth as StreamSpawner>::OutputType) -> Self {
    let cartridge = Cartridge::from_ines_rom(rom);
    let state = NESState::new(cartridge);

    let mut machine = Self {
      state,
      apu_sender,
      last_executed_instruction: None,
      last_disassembly_machine_state: None,
      disassembly_writer: None,
    };

    machine.reset();

    machine
  }

  pub fn execute_frame(&mut self, pixbuf: &mut Pixbuf) {
    loop {
      self.tick(pixbuf);

      if self.state.ppu.cycle == 1 && self.state.ppu.scanline == -1 {
        break;
      }
    }
  }

  pub fn tick_cpu(&mut self) {
    let captured_state = DisassemblyMachineState::capture(
      &self.state.cpu,
      &self.state.ppu,
      self.state.cpu_cycle_count,
      self.state.cartridge.cpu_bus(),
    );

    let executed_instruction = self.state.cpu.tick(self.state.cartridge.cpu_bus_mut());
    self.state.cpu_cycle_count += 1;

    if let Some(instruction) = executed_instruction {
      self.last_disassembly_machine_state = Some(captured_state);
      self.last_executed_instruction = Some(instruction);
    }
  }

  pub fn tick_ppu(&mut self, pixbuf: &mut Pixbuf) {
    let nmi_set = self
      .state
      .ppu
      .tick(pixbuf, self.state.cartridge.ppu_cpu_bus_mut());
    self.state.ppu_cycle_count += 1;

    if nmi_set {
      self.nmi();
    }
  }

  pub fn tick_apu(&mut self) {
    let irq_set = self
      .state
      .cartridge
      .cpu_bus_mut()
      .tick_apu(&self.apu_sender, self.state.cpu_cycle_count);

    if irq_set {
      self.state.cpu.irq_set = true;
    }
  }

  pub fn tick(&mut self, pixbuf: &mut Pixbuf) {
    if self.state.ppu_cycle_count % 3 == 0 {
      let dma_ticked = self
        .state
        .cartridge
        .cpu_bus_mut()
        .maybe_tick_dma(self.state.ppu_cycle_count);

      if !dma_ticked {
        self.log_last_executed_instruction();
        self.tick_cpu();
      }
    }

    self.tick_ppu(pixbuf);
    self.tick_apu();
  }

  fn log_last_executed_instruction(&mut self) {
    if let Some(disassembly_writer) = &mut self.disassembly_writer {
      if let Some(executed_instruction) = &self.last_executed_instruction {
        if let Some(prev_state) = &self.last_disassembly_machine_state {
          if self.state.cpu.wait_cycles == 0 {
            disassembly_writer
              .write()
              .unwrap()
              .write_fmt(format_args!(
                "{}\n",
                executed_instruction.disassemble(prev_state)
              ))
              .unwrap();
          }
        }
      }
    }
  }

  pub fn nmi(&mut self) {
    self.state.cpu.nmi_set = true;
  }

  pub fn reset(&mut self) {
    CPU::reset(self.state.cartridge.cpu_bus_mut(), &mut self.state.cpu);

    self.state.ppu_cycle_count = 0;
    self.state.cpu_cycle_count = 0;
  }
}
