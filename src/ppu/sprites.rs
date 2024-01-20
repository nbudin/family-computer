use bitfield_struct::bitfield;
use bytemuck::{Pod, Zeroable};

use super::{PPUCPUBusTrait, PPU};

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
    &self,
    ppu_cpu_bus: &mut dyn PPUCPUBusTrait,
  ) -> (u8, u8, SpritePriority, bool) {
    let mut fg_pixel: u8 = 0;
    let mut fg_palette: u8 = 0;
    let mut priority: SpritePriority = SpritePriority::Background;
    let mut sprite0 = false;
    let mask = ppu_cpu_bus.ppu_memory_mut().mask();

    if mask.render_sprites() {
      for sprite_index in 0..self.sprite_scanline.len() {
        let sprite = &self.sprite_scanline[sprite_index];

        if sprite.oam_entry.x() == 0 {
          let fg_pixel_low = ((self.sprite_shifter_pattern_low[sprite_index] & 0x80) > 1) as u8;
          let fg_pixel_high = ((self.sprite_shifter_pattern_high[sprite_index] & 0x80) > 1) as u8;
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

  pub fn evaluate_scanline_sprites(&mut self, ppu_cpu_bus: &mut dyn PPUCPUBusTrait) {
    self.sprite_scanline.truncate(0);

    let sprite_height = ppu_cpu_bus.control_mut().sprite_height() as i32;
    for (oam_index, entry) in ppu_cpu_bus.oam_mut().iter().enumerate() {
      let diff = self.scanline - entry.y() as i32;
      if diff >= 0 && diff < sprite_height {
        self.sprite_scanline.push(ActiveSprite {
          oam_entry: *entry,
          oam_index,
        });
      }

      if self.sprite_scanline.len() == 9 {
        break;
      }
    }

    ppu_cpu_bus
      .status_mut()
      .set_sprite_overflow(self.sprite_scanline.len() > 8);
    self.sprite_scanline.truncate(8);
  }

  pub fn load_sprite_data_for_next_scanline(
    &mut self,
    sprite_index: usize,
    ppu_cpu_bus: &mut dyn PPUCPUBusTrait,
  ) {
    let sprite = self.sprite_scanline[sprite_index].clone();

    let sprite_pattern_addr_low = if !ppu_cpu_bus.control_mut().sprite_size() {
      // 8x8 mode
      let sprite_pattern_start_low = ((ppu_cpu_bus.control_mut().pattern_sprite() as u16) << 12)
        | ((sprite.oam_entry.tile_id() as u16) << 4);

      if !sprite.oam_entry.flip_vertical() {
        // sprite is not vertically flipped
        sprite_pattern_start_low | ((self.scanline as u16) - (sprite.oam_entry.y() as u16))
      } else {
        // sprite is vertically flipped
        sprite_pattern_start_low | (7 - ((self.scanline as u16) - (sprite.oam_entry.y() as u16)))
      }
    } else {
      // 8x16 mode
      let top_half = self.scanline - (sprite.oam_entry.y() as i32) < 8;

      if !sprite.oam_entry.flip_vertical() {
        // sprite is not vertically flipped
        if top_half {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | (((sprite.oam_entry.tile_id() as u16) & 0xfe) << 4)
            | (((self.scanline as u16) - (sprite.oam_entry.y() as u16)) & 0x07)
        } else {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | ((((sprite.oam_entry.tile_id() as u16) & 0xfe) + 1) << 4)
            | (((self.scanline as u16) - (sprite.oam_entry.y() as u16)) & 0x07)
        }
      } else {
        // sprite is vertically flipped
        if top_half {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | ((((sprite.oam_entry.tile_id() as u16) & 0xfe) + 1) << 4)
            | ((7 - ((self.scanline as u16) - (sprite.oam_entry.y() as u16))) & 0x07)
        } else {
          (((sprite.oam_entry.tile_id() as u16) & 0x01) << 12)
            | (((sprite.oam_entry.tile_id() as u16) & 0xfe) << 4)
            | ((7 - ((self.scanline as u16) - (sprite.oam_entry.y() as u16))) & 0x07)
        }
      }
    };

    let sprite_pattern_addr_high = sprite_pattern_addr_low + 8;
    let mut sprite_pattern_bits_low = ppu_cpu_bus.ppu_memory_mut().read(sprite_pattern_addr_low);
    let mut sprite_pattern_bits_high = ppu_cpu_bus.ppu_memory_mut().read(sprite_pattern_addr_high);

    if sprite.oam_entry.flip_horizontal() {
      sprite_pattern_bits_low = flip_byte(sprite_pattern_bits_low);
      sprite_pattern_bits_high = flip_byte(sprite_pattern_bits_high);
    }

    self.sprite_shifter_pattern_low[sprite_index] = sprite_pattern_bits_low;
    self.sprite_shifter_pattern_high[sprite_index] = sprite_pattern_bits_high;
  }
}
