use crate::{
  bus::Bus,
  gui::{BYTES_PER_PIXEL, PIXEL_BUFFER_SIZE, PIXEL_BUFFER_WIDTH},
  machine::Machine,
  palette::PALETTE,
};

use super::PPU;

impl PPU {
  pub fn get_palette_color(machine: &Machine, palette_index: u16, color_index: u16) -> u8 {
    machine
      .ppu_memory()
      .read_readonly(0x3f00 + (palette_index * 4) + color_index)
  }

  pub fn get_current_pixel_bg_color(machine: &Machine) -> [u8; 3] {
    let bit_mux = 0x8000 >> machine.ppu.fine_x;

    let plane0_pixel = u16::from((machine.ppu.bg_shifter_pattern_low & bit_mux) > 0);
    let plane1_pixel = u16::from((machine.ppu.bg_shifter_pattern_high & bit_mux) > 0);
    let bg_pixel = (plane1_pixel << 1) | plane0_pixel;

    let plane0_palette: u16 = u16::from((machine.ppu.bg_shifter_attrib_low & bit_mux) > 0);
    let plane1_palette: u16 = u16::from((machine.ppu.bg_shifter_attrib_high & bit_mux) > 0);
    let bg_palette = (plane1_palette << 1) | plane0_palette;

    PALETTE[PPU::get_palette_color(machine, bg_palette, bg_pixel) as usize % 64]
  }

  pub fn set_pixel(pixbuf: &mut [u8; PIXEL_BUFFER_SIZE], color: [u8; 3], x: u32, y: u32) {
    let offset = (x + (y * PIXEL_BUFFER_WIDTH)) * BYTES_PER_PIXEL;
    let pixel = pixbuf
      .get_mut((offset as usize)..((offset + BYTES_PER_PIXEL) as usize))
      .unwrap();
    pixel.copy_from_slice(&[color[0], color[1], color[2], 255]);
  }
}
