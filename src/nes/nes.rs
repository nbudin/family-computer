use std::{
  any::Any,
  fmt::Debug,
  io::Write,
  sync::{Arc, RwLock},
};

use crate::{
  apu::{APUSynth, APU},
  audio::stream_setup::StreamSpawner,
  bus::{Bus, BusInterceptor, RwHandle},
  cartridge::{load_cartridge, BoxCartridge},
  cpu::{CPUBus, ExecutedInstruction, CPU},
  ppu::{PPUCPUBus, PPUMemory, Pixbuf, PPU},
};

use super::{Controller, INESRom, DMA};

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

#[allow(clippy::upper_case_acronyms)]
pub struct NES {
  pub work_ram: WorkRAM,
  pub cartridge: BoxCartridge,
  pub cpu: CPU,
  pub ppu: PPU,
  pub apu: APU,
  pub apu_sender: <APUSynth as StreamSpawner>::OutputType,
  pub controllers: [Controller; 2],
  pub dma: DMA,
  pub cpu_cycle_count: u64,
  pub ppu_cycle_count: u64,
  pub last_executed_instruction: Option<ExecutedInstruction>,
  pub disassembly_writer: Option<Arc<RwLock<dyn DisassemblyWriter + Send + Sync>>>,
}

impl Debug for NES {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Machine")
      .field("cartridge", &self.cartridge)
      .field("cpu", &self.cpu)
      .field("ppu", &self.ppu)
      .field("apu", &self.apu)
      .field("controllers", &self.controllers)
      .field("cpu_cycle_count", &self.cpu_cycle_count)
      .field("ppu_cycle_count", &self.ppu_cycle_count)
      .field("last_executed_instruction", &self.last_executed_instruction)
      .finish_non_exhaustive()
  }
}

impl NES {
  pub fn from_rom(rom: INESRom, apu_sender: <APUSynth as StreamSpawner>::OutputType) -> Self {
    let mut machine = Self {
      work_ram: [0; 2048],
      cartridge: load_cartridge(rom),
      cpu: CPU::new(),
      ppu: PPU::new(),
      apu: APU::new(),
      apu_sender,
      controllers: [Controller::new(), Controller::new()],
      cpu_cycle_count: 0,
      ppu_cycle_count: 0,
      dma: DMA::new(),
      last_executed_instruction: None,
      disassembly_writer: None,
    };

    machine.reset();

    machine
  }

  pub fn execute_frame(&mut self, pixbuf: &mut Pixbuf) {
    loop {
      self.tick(pixbuf);

      if self.ppu.cycle == 1 && self.ppu.scanline == -1 {
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

  pub fn tick_ppu(&mut self, pixbuf: &mut Pixbuf) {
    let nmi_set = PPU::tick(self, pixbuf);
    self.ppu_cycle_count += 1;

    if nmi_set {
      self.nmi();
    }
  }

  pub fn tick_apu(&mut self) {
    let irq_set = APU::tick(self);

    if irq_set {
      self.cpu.irq_set = true;
    }
  }

  pub fn tick(&mut self, pixbuf: &mut Pixbuf) {
    if self.ppu_cycle_count % 3 == 0 {
      if self.dma.transfer {
        if self.dma.dummy {
          if self.ppu_cycle_count % 2 == 1 {
            self.dma.dummy = false;
          }
        } else if self.ppu_cycle_count % 2 == 0 {
          let addr = self.dma.ram_addr();
          let value = self.cpu_bus_mut().as_mut().read(addr);
          self.dma.store_data(value);
        } else {
          self.dma.write_to_ppu(&mut self.ppu.oam);
        }
      } else {
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
        if self.cpu.wait_cycles == 0 {
          disassembly_writer
            .write()
            .unwrap()
            .write_fmt(format_args!("{}\n", executed_instruction.disassemble()))
            .unwrap();
        }
      }
    }
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
    let ppu_memory = PPUMemory {
      mask: self.ppu.mask,
      palette_ram: RwHandle::ReadOnly(&self.ppu.palette_ram),
      name_tables: RwHandle::ReadOnly(&self.ppu.name_tables),
      pattern_tables: RwHandle::ReadOnly(&self.ppu.pattern_tables),
      mirroring: self.cartridge.get_mirroring(),
    };
    let ppu_memory_interceptor = self.cartridge.ppu_memory_interceptor(ppu_memory);

    let ppu_cpu_bus = PPUCPUBus {
      status: RwHandle::ReadOnly(&self.ppu.status),
      mask: RwHandle::ReadOnly(&self.ppu.mask),
      control: RwHandle::ReadOnly(&self.ppu.control),
      data_buffer: RwHandle::ReadOnly(&self.ppu.data_buffer),
      oam: RwHandle::ReadOnly(&self.ppu.oam),
      oam_addr: RwHandle::ReadOnly(&self.ppu.oam_addr),
      vram_addr: RwHandle::ReadOnly(&self.ppu.vram_addr),
      tram_addr: RwHandle::ReadOnly(&self.ppu.tram_addr),
      fine_x: RwHandle::ReadOnly(&self.ppu.fine_x),
      address_latch: RwHandle::ReadOnly(&self.ppu.address_latch),
      status_register_read_this_tick: RwHandle::ReadOnly(&self.ppu.status_register_read_this_tick),
      ppu_memory: ppu_memory_interceptor,
      mirroring: self.cartridge.get_mirroring(),
    };

    let bus = CPUBus {
      controllers: RwHandle::ReadOnly(&self.controllers),
      work_ram: RwHandle::ReadOnly(&self.work_ram),
      mirroring: self.cartridge.get_mirroring(),
      ppu_cpu_bus,
      dma: RwHandle::ReadOnly(&self.dma),
      apu: RwHandle::ReadOnly(&self.apu),
    };
    self.cartridge.cpu_bus_interceptor(bus)
  }

  pub fn cpu_bus_mut<'a>(&'a mut self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let mirroring = self.cartridge.get_mirroring();

    let ppu_memory = PPUMemory {
      mask: self.ppu.mask,
      palette_ram: RwHandle::ReadWrite(&mut self.ppu.palette_ram),
      name_tables: RwHandle::ReadWrite(&mut self.ppu.name_tables),
      pattern_tables: RwHandle::ReadWrite(&mut self.ppu.pattern_tables),
      mirroring: self.cartridge.get_mirroring(),
    };
    let ppu_memory_interceptor = self.cartridge.ppu_memory_interceptor_mut(ppu_memory);

    let ppu_cpu_bus = PPUCPUBus {
      status: RwHandle::ReadWrite(&mut self.ppu.status),
      mask: RwHandle::ReadWrite(&mut self.ppu.mask),
      control: RwHandle::ReadWrite(&mut self.ppu.control),
      data_buffer: RwHandle::ReadWrite(&mut self.ppu.data_buffer),
      oam: RwHandle::ReadWrite(&mut self.ppu.oam),
      oam_addr: RwHandle::ReadWrite(&mut self.ppu.oam_addr),
      vram_addr: RwHandle::ReadWrite(&mut self.ppu.vram_addr),
      tram_addr: RwHandle::ReadWrite(&mut self.ppu.tram_addr),
      fine_x: RwHandle::ReadWrite(&mut self.ppu.fine_x),
      address_latch: RwHandle::ReadWrite(&mut self.ppu.address_latch),
      status_register_read_this_tick: RwHandle::ReadWrite(
        &mut self.ppu.status_register_read_this_tick,
      ),
      ppu_memory: ppu_memory_interceptor,
      mirroring,
    };

    let bus = CPUBus {
      controllers: RwHandle::ReadWrite(&mut self.controllers),
      work_ram: RwHandle::ReadWrite(&mut self.work_ram),
      mirroring,
      dma: RwHandle::ReadWrite(&mut self.dma),
      apu: RwHandle::ReadWrite(&mut self.apu),
      ppu_cpu_bus,
    };
    self.cartridge.cpu_bus_interceptor_mut(bus)
  }

  pub fn ppu_memory<'a>(&'a self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let bus = PPUMemory {
      mask: self.ppu.mask,
      palette_ram: RwHandle::ReadOnly(&self.ppu.palette_ram),
      name_tables: RwHandle::ReadOnly(&self.ppu.name_tables),
      pattern_tables: RwHandle::ReadOnly(&self.ppu.pattern_tables),
      mirroring: self.cartridge.get_mirroring(),
    };
    self.cartridge.ppu_memory_interceptor(bus)
  }

  pub fn ppu_memory_mut<'a>(&'a mut self) -> Box<dyn BusInterceptor<'a, u16> + 'a> {
    let bus = PPUMemory {
      mask: self.ppu.mask,
      palette_ram: RwHandle::ReadWrite(&mut self.ppu.palette_ram),
      name_tables: RwHandle::ReadWrite(&mut self.ppu.name_tables),
      pattern_tables: RwHandle::ReadWrite(&mut self.ppu.pattern_tables),
      mirroring: self.cartridge.get_mirroring(),
    };
    self.cartridge.ppu_memory_interceptor_mut(bus)
  }
}
