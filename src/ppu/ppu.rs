use crate::{
  bus::Bus,
  gui::{PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_SIZE, PIXEL_BUFFER_WIDTH},
  machine::Machine,
  palette::PALETTE,
};

use super::{
  registers::{PPUControlRegister, PPULoopyRegister, PPUMaskRegister, PPUStatusRegister},
  sprites::PPUOAMEntry,
  ActiveSprite, SpritePriority,
};

#[derive(Debug, Clone, Copy)]
pub enum PPUAddressLatch {
  High = 0,
  Low = 1,
}

#[derive(Debug, Clone)]
pub struct PPU {
  pub cycle: i32,
  pub scanline: i32,
  pub status: PPUStatusRegister,
  pub mask: PPUMaskRegister,
  pub control: PPUControlRegister,
  pub vram_addr: PPULoopyRegister,
  pub tram_addr: PPULoopyRegister,
  pub fine_x: u8,
  pub data_buffer: u8,
  pub address_latch: PPUAddressLatch,
  pub palette_ram: [u8; 32],
  pub name_tables: [[u8; 1024]; 2],
  pub pattern_tables: [[u8; 4096]; 2],
  pub oam: [PPUOAMEntry; 64],
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
  pub status_register_read_this_tick: bool,
  pub status_register_read_last_tick: bool,
  pub oam_addr: u8,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      cycle: 0,
      scanline: -1,
      mask: PPUMaskRegister::new(),
      control: PPUControlRegister::new(),
      status: PPUStatusRegister::new(),
      vram_addr: PPULoopyRegister::new(),
      tram_addr: PPULoopyRegister::new(),
      fine_x: 0,
      data_buffer: 0,
      palette_ram: [0; 32],
      address_latch: PPUAddressLatch::High,
      name_tables: [[0; 1024], [0; 1024]],
      pattern_tables: [[0; 4096], [0; 4096]],
      oam: [PPUOAMEntry::from(0); 64],
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
      status_register_read_this_tick: false,
      status_register_read_last_tick: false,
      oam_addr: 0,
    }
  }

  fn update_visible_scanline(state: &mut Machine) {
    if state.ppu.frame_count % 2 == 1 && state.ppu.scanline == 0 && state.ppu.cycle == 0 {
      // Odd frame cycle skip
      if state.ppu.mask.render_background() || state.ppu.mask.render_sprites() {
        state.ppu.cycle = 1;
      }
    }

    if state.ppu.scanline == -1 && state.ppu.cycle == 1 {
      state.ppu.status.set_vertical_blank(false);
      state.ppu.status.set_sprite_zero_hit(false);
      state.ppu.status.set_sprite_overflow(false);

      state.ppu.sprite_shifter_pattern_low = [0; 8];
      state.ppu.sprite_shifter_pattern_high = [0; 8];
    }

    if (state.ppu.cycle >= 2 && state.ppu.cycle < 258)
      || (state.ppu.cycle >= 321 && state.ppu.cycle < 338)
    {
      state.ppu.update_shifters();

      PPU::update_bg_registers(state);

      if state.ppu.cycle == 256 {
        state.ppu.increment_scroll_y();
      }

      if state.ppu.cycle == 257 {
        state.ppu.load_background_shifters();
        state.ppu.transfer_address_x();
      }

      if state.ppu.scanline == -1 && state.ppu.cycle >= 280 && state.ppu.cycle < 305 {
        state.ppu.transfer_address_y();
      }

      if state.ppu.cycle == 338 || state.ppu.cycle == 340 {
        // superfluous reads of tile id at end of scanline
        let addr = 0x2000 | (u16::from(state.ppu.vram_addr) & 0x0fff);
        let next_tile_id = state.ppu_memory_mut().read(addr);
        state.ppu.bg_next_tile_id = next_tile_id;
      }

      // Foreground rendering =========================================================
      if state.ppu.cycle == 257 && state.ppu.scanline >= 0 {
        PPU::evaluate_scanline_sprites(state);
      }

      if state.ppu.cycle == 340 {
        for sprite_index in 0..state.ppu.sprite_scanline.len() {
          PPU::load_sprite_data_for_next_scanline(state, sprite_index);
        }
      }
    }
  }

  fn draw_current_pixel(state: &mut Machine, pixbuf: &mut [u8; 245760]) {
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

    PPU::set_pixel(
      pixbuf,
      color,
      u32::try_from(state.ppu.cycle - 1).unwrap(),
      u32::try_from(state.ppu.scanline).unwrap(),
    );
  }

  fn update_cycle_and_scanline(state: &mut Machine) {
    state.ppu.cycle += 1;
    if state.ppu.cycle >= 341 {
      state.ppu.cycle = 0;
      state.ppu.scanline += 1;
      if state.ppu.scanline >= 261 {
        state.ppu.frame_count += 1;
        state.ppu.scanline = -1;
      }
    }
  }

  pub fn tick(state: &mut Machine, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) -> bool {
    let mut nmi_set = false;
    state.ppu.status_register_read_last_tick = state.ppu.status_register_read_this_tick;
    state.ppu.status_register_read_this_tick = false;

    if state.ppu.scanline >= -1 && state.ppu.scanline < 240 {
      PPU::update_visible_scanline(state);
    }

    if state.ppu.scanline == 240 {
      // post render scanline - do nothing
    }

    if state.ppu.scanline >= 241 && state.ppu.scanline < 261 {
      if state.ppu.scanline == 241 && state.ppu.cycle == 1 {
        // emulate a race condition in the PPU: reading the status register suppresses vblank next tick and nmi this tick
        if !state.ppu.status_register_read_last_tick {
          state.ppu.status.set_vertical_blank(true);
        }
        if !state.ppu.status_register_read_this_tick && state.ppu.control.enable_nmi() {
          nmi_set = true;
        }
      }
    }

    if state.ppu.cycle >= 1
      && state.ppu.scanline >= 0
      && state.ppu.cycle <= PIXEL_BUFFER_WIDTH as i32
      && state.ppu.scanline < PIXEL_BUFFER_HEIGHT as i32
    {
      PPU::draw_current_pixel(state, pixbuf);
    }

    PPU::update_cycle_and_scanline(state);

    nmi_set
  }
}
