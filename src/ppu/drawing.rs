use crate::{bus::Bus, nes::NES};

use super::{
  palette::PALETTE, Pixbuf, SpritePriority, PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_WIDTH, PPU,
};

impl PPU {
  pub fn get_palette_color(nes: &NES, palette_index: u16, color_index: u16) -> u8 {
    nes
      .ppu_memory()
      .read_readonly(0x3f00 + (palette_index * 4) + color_index)
  }

  pub fn get_current_pixel_bg_color_and_palette(nes: &NES) -> (u8, u8) {
    let bit_mux = 0x8000 >> nes.ppu.fine_x;

    let plane0_pixel = u16::from((nes.ppu.bg_shifter_pattern_low & bit_mux) > 0);
    let plane1_pixel = u16::from((nes.ppu.bg_shifter_pattern_high & bit_mux) > 0);
    let bg_pixel = (plane1_pixel << 1) | plane0_pixel;

    let plane0_palette: u16 = u16::from((nes.ppu.bg_shifter_attrib_low & bit_mux) > 0);
    let plane1_palette: u16 = u16::from((nes.ppu.bg_shifter_attrib_high & bit_mux) > 0);
    let bg_palette = (plane1_palette << 1) | plane0_palette;

    (bg_pixel as u8, bg_palette as u8)
  }

  pub fn draw_current_pixel(nes: &mut NES, pixbuf: &mut Pixbuf) {
    let (bg_pixel, bg_palette) = if nes.ppu.mask.render_background() {
      PPU::get_current_pixel_bg_color_and_palette(nes)
    } else {
      (0, 0)
    };

    let (fg_pixel, fg_palette, priority, sprite0) = if nes.ppu.mask.render_sprites() {
      PPU::get_current_pixel_fg_color_palette_priority_and_sprite0(nes)
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
      if sprite0 && nes.ppu.mask.render_background() && nes.ppu.mask.render_sprites() {
        if !(nes.ppu.mask.render_background_left() || nes.ppu.mask.render_sprites_left()) {
          if nes.ppu.cycle >= 9 && nes.ppu.cycle < 258 {
            nes.ppu.status.set_sprite_zero_hit(true);
          }
        } else if nes.ppu.cycle >= 1 && nes.ppu.cycle < 258 {
          nes.ppu.status.set_sprite_zero_hit(true);
        }
      }

      if priority == SpritePriority::Foreground {
        (fg_pixel, fg_palette)
      } else {
        (bg_pixel, bg_palette)
      }
    };

    let color =
      PALETTE[PPU::get_palette_color(nes, palette as u16, pixel as u16) as usize % PALETTE.len()];

    let x = nes.ppu.cycle - 1;
    let y = nes.ppu.scanline;
    if x >= 0 && y >= 0 && x < PIXEL_BUFFER_WIDTH as i32 && y < PIXEL_BUFFER_HEIGHT as i32 {
      pixbuf.set_pixel(color, u32::try_from(x).unwrap(), u32::try_from(y).unwrap());
    }
  }
}
