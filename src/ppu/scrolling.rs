use super::{PPUCPUBusTrait, PPU};

impl PPU {
  pub fn increment_scroll_x(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    let mask = ppu_cpu_bus.ppu_memory_mut().mask();

    if mask.render_background() || mask.render_sprites() {
      let vram_addr = ppu_cpu_bus.vram_addr_mut();

      if vram_addr.coarse_x() == 31 {
        vram_addr.set_coarse_x(0);
        vram_addr.set_nametable_x(!vram_addr.nametable_x());
      } else {
        vram_addr.set_coarse_x(vram_addr.coarse_x() + 1);
      }
    }
  }

  pub fn increment_scroll_y(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    let mask = ppu_cpu_bus.ppu_memory_mut().mask();

    if mask.render_background() || mask.render_sprites() {
      let vram_addr = ppu_cpu_bus.vram_addr_mut();

      if vram_addr.fine_y() < 7 {
        vram_addr.set_fine_y(vram_addr.fine_y() + 1);
      } else {
        vram_addr.set_fine_y(0);

        if vram_addr.coarse_y() == 29 {
          vram_addr.set_coarse_y(0);
          vram_addr.set_nametable_y(!vram_addr.nametable_y());
        } else if vram_addr.coarse_y() == 31 {
          vram_addr.set_coarse_y(0);
        } else {
          vram_addr.set_coarse_y(vram_addr.coarse_y() + 1);
        }
      }
    }
  }

  pub fn transfer_address_x(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    let mask = ppu_cpu_bus.ppu_memory_mut().mask();

    if mask.render_background() || mask.render_sprites() {
      let tram_addr = ppu_cpu_bus.tram_addr().clone();
      let vram_addr = ppu_cpu_bus.vram_addr_mut();

      vram_addr.set_nametable_x(tram_addr.nametable_x());
      vram_addr.set_coarse_x(tram_addr.coarse_x());
    }
  }

  pub fn transfer_address_y(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    let mask = ppu_cpu_bus.ppu_memory_mut().mask();

    if mask.render_background() || mask.render_sprites() {
      let tram_addr = ppu_cpu_bus.tram_addr().clone();
      let vram_addr = ppu_cpu_bus.vram_addr_mut();

      vram_addr.set_fine_y(tram_addr.fine_y());
      vram_addr.set_nametable_y(tram_addr.nametable_y());
      vram_addr.set_coarse_y(tram_addr.coarse_y());
    }
  }

  pub fn load_background_shifters(&mut self) {
    self.bg_shifter_pattern_low =
      (self.bg_shifter_pattern_low & 0xff00) | self.bg_next_tile_low as u16;
    self.bg_shifter_pattern_high =
      (self.bg_shifter_pattern_high & 0xff00) | self.bg_next_tile_high as u16;
    self.bg_shifter_attrib_low = (self.bg_shifter_attrib_low & 0xff00)
      | (if self.bg_next_tile_attrib & 0b01 > 0 {
        0xff
      } else {
        0
      });
    self.bg_shifter_attrib_high = (self.bg_shifter_attrib_high & 0xff00)
      | (if self.bg_next_tile_attrib & 0b10 > 0 {
        0xff
      } else {
        0
      });
  }

  pub fn update_shifters(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    let mask = ppu_cpu_bus.ppu_memory_mut().mask();

    if mask.render_background() {
      self.bg_shifter_pattern_high <<= 1;
      self.bg_shifter_pattern_low <<= 1;
      self.bg_shifter_attrib_high <<= 1;
      self.bg_shifter_attrib_low <<= 1;
    }

    if mask.render_sprites() && self.cycle >= 1 && self.cycle < 258 {
      for sprite_index in 0..self.sprite_scanline.len() {
        let sprite = &mut self.sprite_scanline[sprite_index];
        if sprite.oam_entry.x() > 0 {
          sprite.oam_entry.set_x(sprite.oam_entry.x() - 1);
        } else {
          self.sprite_shifter_pattern_low[sprite_index] <<= 1;
          self.sprite_shifter_pattern_high[sprite_index] <<= 1;
        }
      }
    }
  }

  pub fn update_bg_registers(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    match (self.cycle - 1) % 8 {
      0 => {
        self.load_background_shifters();

        let addr = 0x2000 | (u16::from(*ppu_cpu_bus.vram_addr_mut()) & 0x0fff);
        let next_tile_id = ppu_cpu_bus.ppu_memory_mut().read(addr);
        self.bg_next_tile_id = next_tile_id;
      }
      2 => {
        let addr = 0x23c0
          | (u16::from(ppu_cpu_bus.vram_addr_mut().nametable_y()) << 11)
          | (u16::from(ppu_cpu_bus.vram_addr_mut().nametable_x()) << 10)
          | ((ppu_cpu_bus.vram_addr_mut().coarse_y() as u16 >> 2) << 3)
          | (ppu_cpu_bus.vram_addr_mut().coarse_x() as u16 >> 2);
        let next_tile_attrib = ppu_cpu_bus.ppu_memory_mut().read(addr);
        self.bg_next_tile_attrib = next_tile_attrib;

        if ppu_cpu_bus.vram_addr_mut().coarse_y() & 0x02 > 0 {
          self.bg_next_tile_attrib >>= 4;
        }
        if ppu_cpu_bus.vram_addr_mut().coarse_x() & 0x02 > 0 {
          self.bg_next_tile_attrib >>= 2;
        }
        self.bg_next_tile_attrib &= 0x03;
      }
      4 => {
        let addr = (u16::from(ppu_cpu_bus.control_mut().pattern_background()) << 12)
          + ((self.bg_next_tile_id as u16) << 4)
          + (ppu_cpu_bus.vram_addr_mut().fine_y() as u16);
        let bg_next_tile_low = ppu_cpu_bus.ppu_memory_mut().read(addr);
        self.bg_next_tile_low = bg_next_tile_low;
      }
      6 => {
        let addr = (u16::from(ppu_cpu_bus.control_mut().pattern_background()) << 12)
          + ((self.bg_next_tile_id as u16) << 4)
          + (ppu_cpu_bus.vram_addr_mut().fine_y() as u16)
          + 8;
        let bg_next_tile_high = ppu_cpu_bus.ppu_memory_mut().read(addr);
        self.bg_next_tile_high = bg_next_tile_high;
      }
      7 => {
        self.increment_scroll_x(ppu_cpu_bus);
      }
      _ => {}
    }
  }
}
