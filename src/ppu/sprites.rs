use bitfield_struct::bitfield;
use bytemuck::{Pod, Zeroable};

use crate::{bus::Bus, nes::NES};

use super::PPU;

#[bitfield(u32)]
#[derive(Pod, Zeroable)]
pub struct PPUOAMEntry {
  pub y: u8,
  pub tile_id: u8,
  #[bits(2)]
  pub palette_index: u8,
  #[bits(3)]
  pub _unused: u8,
  pub priority_behind_background: bool,
  pub flip_horizontal: bool,
  pub flip_vertical: bool,
  pub x: u8,
}

#[derive(Debug, Clone)]
pub struct ActiveSprite {
  pub oam_entry: PPUOAMEntry,
  pub oam_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpritePriority {
  Foreground,
  Background,
}

// https://stackoverflow.com/a/2602885
pub fn flip_byte(b: u8) -> u8 {
  let mut b = b;
  b = ((b & 0xf0) >> 4) | ((b & 0x0f) << 4);
  b = ((b & 0xcc) >> 2) | ((b & 0x33) << 2);
  b = ((b & 0xaa) >> 1) | ((b & 0x55) << 1);
  b
}

impl PPU {
  pub fn get_current_pixel_fg_color_palette_priority_and_sprite0(
    nes: &NES,
  ) -> (u8, u8, SpritePriority, bool) {
    let mut fg_pixel: u8 = 0;
    let mut fg_palette: u8 = 0;
    let mut priority: SpritePriority = SpritePriority::Background;
    let mut sprite0 = false;

    if nes.ppu.mask.render_sprites() {
      for sprite_index in 0..nes.ppu.sprite_scanline.len() {
        let sprite = &nes.ppu.sprite_scanline[sprite_index];

        if sprite.oam_entry.x() == 0 {
          let fg_pixel_low = ((nes.ppu.sprite_shifter_pattern_low[sprite_index] & 0x80) > 1) as u8;
          let fg_pixel_high =
            ((nes.ppu.sprite_shifter_pattern_high[sprite_index] & 0x80) > 1) as u8;
          fg_pixel = (fg_pixel_high << 1) | fg_pixel_low;

          fg_palette = sprite.oam_entry.palette_index() + 4;
          priority = if sprite.oam_entry.priority_behind_background() {
            SpritePriority::Background
          } else {
            SpritePriority::Foreground
          };

          if fg_pixel != 0 {
            if sprite.oam_index == 0 {
              sprite0 = true;
            }

            break;
          }
        }
      }
    }

    (fg_pixel, fg_palette, priority, sprite0)
  }

  pub fn evaluate_scanline_sprites(nes: &mut NES) {
    nes.ppu.sprite_scanline.truncate(0);

    for (oam_index, entry) in nes.ppu.oam.iter().enumerate() {
      let diff = nes.ppu.scanline - entry.y() as i32;
      if diff >= 0 && diff < nes.ppu.control.sprite_height().into() {
        nes.ppu.sprite_scanline.push(ActiveSprite {
          oam_entry: *entry,
          oam_index,
        });
      }

      if nes.ppu.sprite_scanline.len() == 9 {
        break;
      }
    }

    nes
      .ppu
      .status
      .set_sprite_overflow(nes.ppu.sprite_scanline.len() > 8);
    nes.ppu.sprite_scanline.truncate(8);
  }

  pub fn load_sprite_data_for_next_scanline(nes: &mut NES, sprite_index: usize) {
    let sprite = nes.ppu.sprite_scanline[sprite_index].clone();

    let sprite_pattern_addr_low = if !nes.ppu.control.sprite_size() {
      // 8x8 mode
      let sprite_pattern_start_low = ((nes.ppu.control.pattern_sprite() as u16) << 12)
        | ((sprite.oam_entry.tile_id() as u16) << 4);

      if !sprite.oam_entry.flip_vertical() {
        // sprite is not vertically flipped
        sprite_pattern_start_low | ((nes.ppu.scanline as u16) - (sprite.oam_entry.y() as u16))
      } else {
        // sprite is vertically flipped
        sprite_pattern_start_low | (7 - ((nes.ppu.scanline as u16) - (sprite.oam_entry.y() as u16)))
      }
    } else {
      // 8x16 mode
      let top_half = nes.ppu.scanline - (sprite.oam_entry.y() as i32) < 8;

      if !sprite.oam_entry.flip_vertical() {
        // sprite is not vertically flipped
        if top_half {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | (((sprite.oam_entry.tile_id() as u16) & 0xfe) << 4)
            | (((nes.ppu.scanline as u16) - (sprite.oam_entry.y() as u16)) & 0x07)
        } else {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | ((((sprite.oam_entry.tile_id() as u16) & 0xfe) + 1) << 4)
            | (((nes.ppu.scanline as u16) - (sprite.oam_entry.y() as u16)) & 0x07)
        }
      } else {
        // sprite is vertically flipped
        if top_half {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | ((((sprite.oam_entry.tile_id() as u16) & 0xfe) + 1) << 4)
            | ((7 - ((nes.ppu.scanline as u16) - (sprite.oam_entry.y() as u16))) & 0x07)
        } else {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | (((sprite.oam_entry.tile_id() as u16) & 0xfe) << 4)
            | ((7 - ((nes.ppu.scanline as u16) - (sprite.oam_entry.y() as u16))) & 0x07)
        }
      }
    };

    let sprite_pattern_addr_high = sprite_pattern_addr_low + 8;
    let mut sprite_pattern_bits_low = nes.ppu_memory_mut().read(sprite_pattern_addr_low);
    let mut sprite_pattern_bits_high = nes.ppu_memory_mut().read(sprite_pattern_addr_high);

    if sprite.oam_entry.flip_horizontal() {
      sprite_pattern_bits_low = flip_byte(sprite_pattern_bits_low);
      sprite_pattern_bits_high = flip_byte(sprite_pattern_bits_high);
    }

    nes.ppu.sprite_shifter_pattern_low[sprite_index] = sprite_pattern_bits_low;
    nes.ppu.sprite_shifter_pattern_high[sprite_index] = sprite_pattern_bits_high;
  }
}
