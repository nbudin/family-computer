use crate::bus::Bus;

use super::{
  palette::PALETTE, Pixbuf, SpritePriority, PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_WIDTH, PPU,
};

impl PPU {
  pub fn get_palette_color(palette_index: u16, color_index: u16, ppu_memory: &dyn Bus<u16>) -> u8 {
    ppu_memory.read_readonly(0x3f00 + (palette_index * 4) + color_index)
  }

  pub fn get_current_pixel_bg_color_and_palette(&self) -> (u8, u8) {
    let bit_mux = 0x8000 >> self.fine_x;

    let plane0_pixel = u16::from((self.bg_shifter_pattern_low & bit_mux) > 0);
    let plane1_pixel = u16::from((self.bg_shifter_pattern_high & bit_mux) > 0);
    let bg_pixel = (plane1_pixel << 1) | plane0_pixel;

    let plane0_palette: u16 = u16::from((self.bg_shifter_attrib_low & bit_mux) > 0);
    let plane1_palette: u16 = u16::from((self.bg_shifter_attrib_high & bit_mux) > 0);
    let bg_palette = (plane1_palette << 1) | plane0_palette;

    (bg_pixel as u8, bg_palette as u8)
  }

  pub fn draw_current_pixel(&mut self, pixbuf: &mut Pixbuf, ppu_memory: &dyn Bus<u16>) {
    let (bg_pixel, bg_palette) = if self.mask.render_background() {
      self.get_current_pixel_bg_color_and_palette()
    } else {
      (0, 0)
    };

    let (fg_pixel, fg_palette, priority, sprite0) = if self.mask.render_sprites() {
      self.get_current_pixel_fg_color_palette_priority_and_sprite0()
    } else {
      (0, 0, SpritePriority::Background, false)
    };

    let (pixel, palette) = if bg_pixel == 0 && fg_pixel == 0 {
      (0, 0)
    } else if bg_pixel == 0 {
      (fg_pixel, fg_palette)
    } else if fg_pixel == 0 {
      (bg_pixel, bg_palette)
    } else {
      if sprite0 && self.mask.render_background() && self.mask.render_sprites() {
        if !(self.mask.render_background_left() || self.mask.render_sprites_left()) {
          if self.cycle >= 9 && self.cycle < 258 {
            self.status.set_sprite_zero_hit(true);
          }
        } else if self.cycle >= 1 && self.cycle < 258 {
          self.status.set_sprite_zero_hit(true);
        }
      }

      if priority == SpritePriority::Foreground {
        (fg_pixel, fg_palette)
      } else {
        (bg_pixel, bg_palette)
      }
    };

    let color = PALETTE
      [PPU::get_palette_color(palette as u16, pixel as u16, ppu_memory) as usize % PALETTE.len()];

    let x = self.cycle - 1;
    let y = self.scanline;
    if x >= 0 && y >= 0 && x < PIXEL_BUFFER_WIDTH as i32 && y < PIXEL_BUFFER_HEIGHT as i32 {
      pixbuf.set_pixel(color, u32::try_from(x).unwrap(), u32::try_from(y).unwrap());
    }
  }
}
