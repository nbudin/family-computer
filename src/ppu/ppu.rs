use super::{ActiveSprite, PPUCPUBusTrait, Pixbuf};

#[derive(Debug, Clone, Copy)]
pub enum PPUAddressLatch {
  High = 0,
  Low = 1,
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
  pub cycle: i32,
  pub scanline: i32,
  pub sprite_scanline: Vec<ActiveSprite>,
  pub bg_next_tile_id: u8,
  pub bg_next_tile_attrib: u8,
  pub bg_next_tile_low: u8,
  pub bg_next_tile_high: u8,
  pub bg_shifter_pattern_low: u16,
  pub bg_shifter_pattern_high: u16,
  pub bg_shifter_attrib_low: u16,
  pub bg_shifter_attrib_high: u16,
  pub sprite_shifter_pattern_low: [u8; 8],
  pub sprite_shifter_pattern_high: [u8; 8],
  pub frame_count: u64,
  pub status_register_read_last_tick: bool,
}

impl Default for PPU {
  fn default() -> Self {
    Self::new()
  }
}

impl PPU {
  pub fn new() -> Self {
    Self {
      cycle: 0,
      scanline: -1,
      sprite_scanline: Vec::with_capacity(8),
      bg_next_tile_attrib: 0,
      bg_next_tile_id: 0,
      bg_next_tile_low: 0,
      bg_next_tile_high: 0,
      bg_shifter_attrib_high: 0,
      bg_shifter_attrib_low: 0,
      bg_shifter_pattern_high: 0,
      bg_shifter_pattern_low: 0,
      sprite_shifter_pattern_low: [0; 8],
      sprite_shifter_pattern_high: [0; 8],
      frame_count: 0,
      status_register_read_last_tick: false,
    }
  }

  fn start_frame(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    let status = ppu_cpu_bus.status_mut();
    status.set_vertical_blank(false);
    status.set_sprite_zero_hit(false);
    status.set_sprite_overflow(false);

    self.sprite_shifter_pattern_low = [0; 8];
    self.sprite_shifter_pattern_high = [0; 8];
  }

  fn update_registers_on_renderable_scanline(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    if (self.cycle >= 1 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
      self.update_shifters(ppu_cpu_bus);
      self.update_bg_registers(ppu_cpu_bus);
    }

    if self.cycle == 256 {
      self.increment_scroll_y(ppu_cpu_bus);
    }

    if self.cycle == 257 {
      self.load_background_shifters();
      self.transfer_address_x(ppu_cpu_bus);
    }

    if self.cycle == 338 || self.cycle == 340 {
      // superfluous reads of tile id at end of scanline
      let addr = 0x2000 | (u16::from(*ppu_cpu_bus.vram_addr_mut()) & 0x0fff);
      let next_tile_id = ppu_cpu_bus.ppu_memory_mut().read(addr);
      self.bg_next_tile_id = next_tile_id;
    }

    if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
      self.transfer_address_y(ppu_cpu_bus);
    }

    // Foreground rendering =========================================================
    if self.cycle == 257 && self.scanline >= 0 {
      self.evaluate_scanline_sprites(ppu_cpu_bus);
    }

    if self.cycle == 340 {
      for sprite_index in 0..self.sprite_scanline.len() {
        self.load_sprite_data_for_next_scanline(sprite_index, ppu_cpu_bus);
      }
    }
  }

  fn increment_cycle_and_scanline(&mut self) {
    self.cycle += 1;

    if self.cycle >= 341 {
      self.cycle = 0;
      self.scanline += 1;

      if self.scanline >= 261 {
        self.scanline = -1;
        self.frame_count += 1;
      }
    }
  }

  pub fn tick(&mut self, pixbuf: &mut Pixbuf, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) -> bool {
    let mut trigger_nmi = false;
    self.status_register_read_last_tick = ppu_cpu_bus.get_status_register_read_this_tick();
    ppu_cpu_bus.set_status_register_read_this_tick(false);

    if self.scanline >= -1 && self.scanline < 240 {
      if self.scanline == 0 && self.cycle == 0 && self.frame_count % 2 == 1 {
        // Odd frame cycle skip
        let mask = ppu_cpu_bus.ppu_memory_mut().mask();
        if mask.render_background() || mask.render_sprites() {
          self.cycle = 1;
        }
      }

      if self.scanline == -1 && self.cycle == 1 {
        self.start_frame(ppu_cpu_bus);
      }

      self.update_registers_on_renderable_scanline(ppu_cpu_bus);
    }

    if self.scanline == 241 && self.cycle == 1 {
      // emulate a race condition in the PPU: reading the status register suppresses vblank next tick and nmi this tick
      if !self.status_register_read_last_tick {
        ppu_cpu_bus.status_mut().set_vertical_blank(true);
      }

      if ppu_cpu_bus.control_mut().enable_nmi() && !ppu_cpu_bus.get_status_register_read_this_tick()
      {
        trigger_nmi = true;
      }
    }

    self.draw_current_pixel(pixbuf, ppu_cpu_bus);
    self.increment_cycle_and_scanline();

    trigger_nmi
  }
}
