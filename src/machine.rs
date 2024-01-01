use std::{any::Any, fmt::Debug, io::Write};

use crate::{
  bus_interceptor::BusInterceptor,
  cartridge::{load_cartridge, BoxCartridge},
  controller::Controller,
  cpu::{CPUBus, ExecutedInstruction, CPU},
  gui::PIXEL_BUFFER_SIZE,
  ines_rom::INESRom,
  ppu::{PPUMemory, PPU},
  rw_handle::RwHandle,
};

pub type WorkRAM = [u8; 2048];

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

pub struct Machine {
  pub work_ram: WorkRAM,
  pub cartridge: BoxCartridge,
  pub cpu: CPU,
  pub ppu: PPU,
  pub controllers: [Controller; 2],
  pub cpu_cycle_count: u64,
  pub ppu_cycle_count: u64,
  pub last_executed_instruction: Option<ExecutedInstruction>,
  pub disassembly_writer: Option<Box<dyn DisassemblyWriter>>,
}

impl Debug for Machine {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Machine")
      .field("cartridge", &self.cartridge)
      .field("cpu_state", &self.cpu)
      .field("ppu_state", &self.ppu)
      .field("controllers", &self.controllers)
      .field("cpu_cycle_count", &self.cpu_cycle_count)
      .field("ppu_cycle_count", &self.ppu_cycle_count)
      .field("last_executed_instruction", &self.last_executed_instruction)
      .finish_non_exhaustive()
  }
}

impl Clone for Machine {
  fn clone(&self) -> Self {
    Self {
      work_ram: self.work_ram.clone(),
      cartridge: dyn_clone::clone_box(&*self.cartridge),
      cpu: self.cpu.clone(),
      ppu: self.ppu.clone(),
      controllers: self.controllers.clone(),
      cpu_cycle_count: self.cpu_cycle_count.clone(),
      ppu_cycle_count: self.ppu_cycle_count.clone(),
      last_executed_instruction: None,
      disassembly_writer: None,
    }
  }
}

impl Machine {
  pub fn from_rom(rom: INESRom) -> Self {
    let mut machine = Self {
      work_ram: [0; 2048],
      cartridge: load_cartridge(rom),
      cpu: CPU::new(),
      ppu: PPU::new(),
      controllers: [Controller::new(), Controller::new()],
      cpu_cycle_count: 0,
      ppu_cycle_count: 0,
      last_executed_instruction: None,
      disassembly_writer: None,
    };

    machine.reset();

    machine
  }

  pub fn execute_frame(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    loop {
      self.tick(pixbuf);

      if self.ppu.cycle == 1 && self.ppu.scanline == -1 {
        // println!("PPU cycles: {}", self.ppu_cycle_count);
        // panic!("O noes i am ded");
        break;
      }
    }
  }

  pub fn tick_cpu(&mut self) {
    let executed_instruction = CPU::tick(self);
    self.cpu_cycle_count += 1;

    if let Some(instruction) = executed_instruction {
      self.last_executed_instruction = Some(instruction);
    }
  }

  pub fn tick_ppu(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    let nmi_set = PPU::tick(self, pixbuf);
    self.ppu_cycle_count += 1;

    if nmi_set {
      self.nmi();
    }
  }

  pub fn tick(&mut self, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    if self.ppu_cycle_count % 3 == 0 {
      if let Some(disassembly_writer) = &mut self.disassembly_writer {
        if let Some(executed_instruction) = &self.last_executed_instruction {
          if self.cpu.wait_cycles == 0 {
            disassembly_writer
              .write_fmt(format_args!("{}\n", executed_instruction.disassemble()))
              .unwrap();
          }
        }
      }

      self.tick_cpu();
    }

    self.tick_ppu(pixbuf);
  }

  pub fn nmi(&mut self) {
    self.cpu.nmi_set = true;
  }

  pub fn reset(&mut self) {
    CPU::reset(self);
    self.ppu_cycle_count = 0;
    self.cpu_cycle_count = 0;
  }

  pub fn cpu_bus<'a>(&'a self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let bus = CPUBus {
      controllers: RwHandle::ReadOnly(&self.controllers),
      work_ram: RwHandle::ReadOnly(&self.work_ram),
      mirroring: self.cartridge.get_mirroring(),
      ppu: RwHandle::ReadOnly(&self.ppu),
    };
    self.cartridge.cpu_bus_interceptor(bus)
  }

  pub fn cpu_bus_mut<'a>(&'a mut self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let cartridge = &mut self.cartridge;
    let bus = CPUBus {
      controllers: RwHandle::ReadWrite(&mut self.controllers),
      work_ram: RwHandle::ReadWrite(&mut self.work_ram),
      mirroring: cartridge.get_mirroring(),
      ppu: RwHandle::ReadWrite(&mut self.ppu),
    };
    cartridge.cpu_bus_interceptor_mut(bus)
  }

  pub fn ppu_memory<'a>(&'a self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let bus = PPUMemory {
      ppu: RwHandle::ReadOnly(&self.ppu),
      mirroring: self.cartridge.get_mirroring(),
    };
    self.cartridge.ppu_memory_interceptor(bus)
  }

  pub fn ppu_memory_mut<'a>(&'a mut self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let bus = PPUMemory {
      ppu: RwHandle::ReadWrite(&mut self.ppu),
      mirroring: self.cartridge.get_mirroring(),
    };
    self.cartridge.ppu_memory_interceptor_mut(bus)
  }
}
