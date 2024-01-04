use crate::{bus::Bus, machine::Machine};

use super::{
  palette::PALETTE, Pixbuf, SpritePriority, PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_WIDTH, PPU,
};

impl PPU {
  pub fn get_palette_color(machine: &Machine, palette_index: u16, color_index: u16) -> u8 {
    machine
      .ppu_memory()
      .read_readonly(0x3f00 + (palette_index * 4) + color_index)
  }

  pub fn get_current_pixel_bg_color_and_palette(machine: &Machine) -> (u8, u8) {
    let bit_mux = 0x8000 >> machine.ppu.fine_x;

    let plane0_pixel = u16::from((machine.ppu.bg_shifter_pattern_low & bit_mux) > 0);
    let plane1_pixel = u16::from((machine.ppu.bg_shifter_pattern_high & bit_mux) > 0);
    let bg_pixel = (plane1_pixel << 1) | plane0_pixel;

    let plane0_palette: u16 = u16::from((machine.ppu.bg_shifter_attrib_low & bit_mux) > 0);
    let plane1_palette: u16 = u16::from((machine.ppu.bg_shifter_attrib_high & bit_mux) > 0);
    let bg_palette = (plane1_palette << 1) | plane0_palette;

    (bg_pixel as u8, bg_palette as u8)
  }

  pub fn draw_current_pixel(state: &mut Machine, pixbuf: &mut Pixbuf) {
    let (bg_pixel, bg_palette) = if state.ppu.mask.render_background() {
      PPU::get_current_pixel_bg_color_and_palette(state)
    } else {
      (0, 0)
    };

    let (fg_pixel, fg_palette, priority, sprite0) = if state.ppu.mask.render_sprites() {
      PPU::get_current_pixel_fg_color_palette_priority_and_sprite0(state)
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
      if sprite0 {
        if state.ppu.mask.render_background() && state.ppu.mask.render_sprites() {
          if !(state.ppu.mask.render_background_left() || state.ppu.mask.render_sprites_left()) {
            if state.ppu.cycle >= 9 && state.ppu.cycle < 258 {
              state.ppu.status.set_sprite_zero_hit(true);
            }
          } else {
            if state.ppu.cycle >= 1 && state.ppu.cycle < 258 {
              state.ppu.status.set_sprite_zero_hit(true);
            }
          }
        }
      }

      if priority == SpritePriority::Foreground {
        (fg_pixel, fg_palette)
      } else {
        (bg_pixel, bg_palette)
      }
    };

    let color =
      PALETTE[PPU::get_palette_color(state, palette as u16, pixel as u16) as usize % PALETTE.len()];

    let x = state.ppu.cycle - 1;
    let y = state.ppu.scanline;
    if x >= 0 && y >= 0 && x < PIXEL_BUFFER_WIDTH as i32 && y < PIXEL_BUFFER_HEIGHT as i32 {
      pixbuf.set_pixel(color, u32::try_from(x).unwrap(), u32::try_from(y).unwrap());
    }
  }
}
