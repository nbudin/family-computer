use crate::{
  gfx::crt_screen::{BYTES_PER_PIXEL, PIXEL_BUFFER_WIDTH},
  machine::Machine,
  palette::PALETTE,
};

use super::PPU;

impl PPU {
  pub fn get_palette_color(&self, machine: &Machine, palette_index: u16, color_index: u16) -> u8 {
    self.get_ppu_mem(machine, 0x3f00 + (palette_index * 4) + color_index)
  }

  pub fn get_current_pixel_bg_color(&self, machine: &Machine) -> [u8; 3] {
    let bit_mux = 0x8000 >> self.fine_x;

    let plane0_pixel: u16 = if (self.bg_shifter_pattern_low & bit_mux) > 0 {
      1
    } else {
      0
    };
    let plane1_pixel: u16 = if (self.bg_shifter_pattern_high & bit_mux) > 0 {
      0b10
    } else {
      0
    };
    let bg_pixel = plane1_pixel | plane0_pixel;

    let plane0_palette: u16 = if (self.bg_shifter_attrib_low & bit_mux) > 0 {
      1
    } else {
      0
    };
    let plane1_palette: u16 = if (self.bg_shifter_attrib_high & bit_mux) > 0 {
      0b10
    } else {
      0
    };
    let bg_palette = plane1_palette | plane0_palette;

    PALETTE[self.get_palette_color(machine, bg_palette, bg_pixel) as usize % 64]
  }

  pub fn set_pixel(&mut self, pixbuf: &mut [u8; 245760], color: [u8; 3], x: u32, y: u32) {
    let offset = (x + (y * PIXEL_BUFFER_WIDTH)) * BYTES_PER_PIXEL;
    let pixel = pixbuf
      .get_mut((offset as usize)..((offset + BYTES_PER_PIXEL) as usize))
      .unwrap();
    pixel.copy_from_slice(&[color[0], color[1], color[2], 255]);
  }
}
