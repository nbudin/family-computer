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
  }
}
