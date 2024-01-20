use crate::bus::Bus;

use super::PPU;

impl PPU {
  pub fn increment_scroll_x(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      if self.vram_addr.coarse_x() == 31 {
        self.vram_addr.set_coarse_x(0);
        self
          .vram_addr
          .set_nametable_x(!self.vram_addr.nametable_x());
      } else {
        self.vram_addr.set_coarse_x(self.vram_addr.coarse_x() + 1);
      }
    }
  }

  pub fn increment_scroll_y(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      if self.vram_addr.fine_y() < 7 {
        self.vram_addr.set_fine_y(self.vram_addr.fine_y() + 1);
      } else {
        self.vram_addr.set_fine_y(0);

        if self.vram_addr.coarse_y() == 29 {
          self.vram_addr.set_coarse_y(0);
          self
            .vram_addr
            .set_nametable_y(!self.vram_addr.nametable_y());
        } else if self.vram_addr.coarse_y() == 31 {
          self.vram_addr.set_coarse_y(0);
        } else {
          self.vram_addr.set_coarse_y(self.vram_addr.coarse_y() + 1);
        }
      }
    }
  }

  pub fn transfer_address_x(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
      self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
    }
  }

  pub fn transfer_address_y(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      self.vram_addr.set_fine_y(self.tram_addr.fine_y());
      self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
      self.vram_addr.set_coarse_y(self.tram_addr.coarse_y());
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

  pub fn update_shifters(&mut self) {
    if self.mask.render_background() {
      self.bg_shifter_pattern_high <<= 1;
      self.bg_shifter_pattern_low <<= 1;
      self.bg_shifter_attrib_high <<= 1;
      self.bg_shifter_attrib_low <<= 1;
    }

    if self.mask.render_sprites() && self.cycle >= 1 && self.cycle < 258 {
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

  pub fn update_bg_registers(&mut self, ppu_memory: &mut dyn Bus<u16>) {
    match (self.cycle - 1) % 8 {
      0 => {
        self.load_background_shifters();

        let addr = 0x2000 | (u16::from(self.vram_addr) & 0x0fff);
        let next_tile_id = ppu_memory.read(addr);
        self.bg_next_tile_id = next_tile_id;
      }
      2 => {
        let addr = 0x23c0
          | (u16::from(self.vram_addr.nametable_y()) << 11)
          | (u16::from(self.vram_addr.nametable_x()) << 10)
          | ((self.vram_addr.coarse_y() as u16 >> 2) << 3)
          | (self.vram_addr.coarse_x() as u16 >> 2);
        let next_tile_attrib = ppu_memory.read(addr);
        self.bg_next_tile_attrib = next_tile_attrib;

        if self.vram_addr.coarse_y() & 0x02 > 0 {
          self.bg_next_tile_attrib >>= 4;
        }
        if self.vram_addr.coarse_x() & 0x02 > 0 {
          self.bg_next_tile_attrib >>= 2;
        }
        self.bg_next_tile_attrib &= 0x03;
      }
      4 => {
        let addr = (u16::from(self.control.pattern_background()) << 12)
          + ((self.bg_next_tile_id as u16) << 4)
          + (self.vram_addr.fine_y() as u16);
        let bg_next_tile_low = ppu_memory.read(addr);
        self.bg_next_tile_low = bg_next_tile_low;
      }
      6 => {
        let addr = (u16::from(self.control.pattern_background()) << 12)
          + ((self.bg_next_tile_id as u16) << 4)
          + (self.vram_addr.fine_y() as u16)
          + 8;
        let bg_next_tile_high = ppu_memory.read(addr);
        self.bg_next_tile_high = bg_next_tile_high;
      }
      7 => {
        self.increment_scroll_x();
      }
      _ => {}
    }
  }
}
