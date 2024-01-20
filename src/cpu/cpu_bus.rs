use crate::{
  apu::{APUSynth, APU},
  audio::stream_setup::StreamSpawner,
  bus::Bus,
  cartridge::bus_interceptor::BusInterceptor,
  nes::{Controller, ControllerButton, DMA},
  ppu::{PPUCPUBus, PPUMemory, PPURegister},
};

pub trait CPUBusTrait: Bus<u16> {
  fn maybe_tick_dma(&mut self, ppu_cycle_count: u64) -> bool;
  fn tick_apu(
    &mut self,
    sender: &<APUSynth as StreamSpawner>::OutputType,
    cpu_cycle_count: u64,
  ) -> bool;
  fn set_controller_button_state(
    &mut self,
    controller_index: usize,
    button: ControllerButton,
    pressed: bool,
  );
}

#[derive(Debug, Clone)]
pub struct CPUBus<I: BusInterceptor<u16, BusType = PPUMemory>> {
  pub work_ram: [u8; 2048],
  pub controllers: [Controller; 2],
  pub ppu_cpu_bus: Box<PPUCPUBus<I>>,
  pub dma: DMA,
  pub apu: APU,
}

impl<I: BusInterceptor<u16, BusType = PPUMemory> + Clone> CPUBus<I> {
  pub fn new(ppu_cpu_bus: PPUCPUBus<I>) -> Self {
    Self {
      work_ram: [0; 2048],
      controllers: [Controller::new(), Controller::new()],
      ppu_cpu_bus: Box::new(ppu_cpu_bus),
      dma: DMA::new(),
      apu: APU::new(),
    }
  }
}

impl<I: BusInterceptor<u16, BusType = PPUMemory> + Clone> CPUBusTrait for CPUBus<I> {
  fn maybe_tick_dma(&mut self, ppu_cycle_count: u64) -> bool {
    if self.dma.transfer {
      if self.dma.dummy {
        if ppu_cycle_count % 2 == 1 {
          self.dma.dummy = false;
        }
      } else if ppu_cycle_count % 2 == 0 {
        let addr = self.dma.ram_addr();
        let value = self.read(addr);
        self.dma.store_data(value);
      } else {
        let oam = &mut self.ppu_cpu_bus.oam;
        self.dma.write_to_ppu(oam);
      }

      true
    } else {
      false
    }
  }

  fn tick_apu(
    &mut self,
    sender: &<APUSynth as StreamSpawner>::OutputType,
    cpu_cycle_count: u64,
  ) -> bool {
    APU::tick(&mut self.apu, sender, cpu_cycle_count)
  }

  fn set_controller_button_state(
    &mut self,
    controller_index: usize,
    button: ControllerButton,
    pressed: bool,
  ) {
    self.controllers[controller_index].set_button_state(button, pressed)
  }
}

impl<I: BusInterceptor<u16, BusType = PPUMemory> + Clone> Bus<u16> for CPUBus<I> {
  fn try_read_readonly(&self, addr: u16) -> Option<u8> {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      Some(self.work_ram[usize::from(actual_address)])
    } else if addr < 0x4000 {
      self
        .ppu_cpu_bus
        .try_read_readonly(PPURegister::from_address(addr))
    } else if addr == 0x4014 {
      None
    } else if addr < 0x4016 {
      self.apu.try_read_readonly(addr)
    } else if addr < 0x4018 {
      let controller = &self.controllers[addr as usize - 0x4016];
      Some(controller.read_readonly(()))
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      None
    } else {
      None
    }
  }

  fn read_side_effects(&mut self, addr: u16) {
    if addr < 0x2000 {
    } else if addr < 0x4000 {
      self
        .ppu_cpu_bus
        .read_side_effects(PPURegister::from_address(addr))
    } else if addr == 0x4014 {
    } else if addr < 0x4016 {
      self.apu.read_side_effects(addr)
    } else if addr < 0x4018 {
      let controller = &mut self.controllers[addr as usize - 0x4016];
      controller.read_side_effects(())
    } else if addr < 0x4020 {
      // TODO: CPU test mode
    }
  }

  fn write(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      let mut work_ram = self.work_ram;
      work_ram[usize::from(actual_address)] = value;
    } else if addr < 0x4000 {
      self
        .ppu_cpu_bus
        .write(PPURegister::from_address(addr), value);
    } else if addr == 0x4014 {
      self.dma.page = value;
      self.dma.addr = 0;
      self.dma.transfer = true;
    } else if addr < 0x4016 {
      self.apu.write(addr, value)
    } else if addr < 0x4018 {
      let controller_index = addr as usize - 0x4016;
      let controller = &mut self.controllers[controller_index];
      controller.poll();
    } else if addr < 0x4020 {
      // TODO: CPU test mode
    }
  }
}
