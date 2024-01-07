use crate::{
  apu::APU,
  bus::{Bus, RwHandle},
  cartridge::CartridgeMirroring,
  controller::Controller,
  dma::DMA,
  ppu::{PPUCPUBus, PPURegister, PPU},
};

pub struct CPUBus<'a> {
  pub work_ram: RwHandle<'a, [u8; 2048]>,
  pub controllers: RwHandle<'a, [Controller; 2]>,
  pub ppu: RwHandle<'a, PPU>,
  pub dma: RwHandle<'a, DMA>,
  pub apu: RwHandle<'a, APU>,
  pub mirroring: CartridgeMirroring,
}

impl Bus<u16> for CPUBus<'_> {
  fn try_read_readonly(&self, addr: u16) -> Option<u8> {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      Some(self.work_ram[usize::from(actual_address)])
    } else if addr < 0x4000 {
      let ppu_cpu_bus = PPUCPUBus {
        mirroring: self.mirroring,
        ppu: RwHandle::ReadOnly(&self.ppu),
      };
      ppu_cpu_bus.try_read_readonly(PPURegister::from_address(addr))
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
      let mut ppu_cpu_bus = PPUCPUBus {
        mirroring: self.mirroring,
        ppu: RwHandle::ReadWrite(self.ppu.get_mut()),
      };
      ppu_cpu_bus.read_side_effects(PPURegister::from_address(addr))
    } else if addr == 0x4014 {
    } else if addr < 0x4016 {
      let apu = self.apu.get_mut();
      apu.read_side_effects(addr)
    } else if addr < 0x4018 {
      let controller = &mut self.controllers.get_mut()[addr as usize - 0x4016];
      controller.read_side_effects(())
    } else if addr < 0x4020 {
      // TODO: CPU test mode
    } else {
    }
  }

  fn write(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      let actual_address = addr % 0x800;
      let work_ram = self.work_ram.get_mut();
      work_ram[usize::from(actual_address)] = value;
    } else if addr < 0x4000 {
      let mut ppu_cpu_bus = PPUCPUBus {
        mirroring: self.mirroring,
        ppu: RwHandle::ReadWrite(self.ppu.get_mut()),
      };
      ppu_cpu_bus.write(PPURegister::from_address(addr), value);
    } else if addr == 0x4014 {
      let dma = self.dma.get_mut();
      dma.page = value;
      dma.addr = 0;
      dma.transfer = true;
    } else if addr < 0x4016 {
      let apu = self.apu.get_mut();
      apu.write(addr, value)
    } else if addr < 0x4018 {
      let controller_index = addr as usize - 0x4016;
      let controller = &mut self.controllers.get_mut()[controller_index];
      controller.poll();
    } else if addr < 0x4020 {
      // TODO: CPU test mode
      ()
    }
  }
}
