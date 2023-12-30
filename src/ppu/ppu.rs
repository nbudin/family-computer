use crate::{
  gui::{PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_SIZE, PIXEL_BUFFER_WIDTH},
  machine::Machine,
};

use super::registers::{PPUControlRegister, PPULoopyRegister, PPUMaskRegister, PPUStatusRegister};

#[derive(Debug, Clone)]
pub enum PPUAddressLatch {
  High,
  Low,
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
  pub bg_next_tile_id: u8,
  pub bg_next_tile_attrib: u8,
  pub bg_next_tile_low: u8,
  pub bg_next_tile_high: u8,
  pub bg_shifter_pattern_low: u16,
  pub bg_shifter_pattern_high: u16,
  pub bg_shifter_attrib_low: u16,
  pub bg_shifter_attrib_high: u16,
  pub frame_count: u64,
  pub status_register_read_this_tick: bool,
  pub status_register_read_last_tick: bool,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      cycle: 0,
      scanline: 0,
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
      bg_next_tile_attrib: 0,
      bg_next_tile_id: 0,
      bg_next_tile_low: 0,
      bg_next_tile_high: 0,
      bg_shifter_attrib_high: 0,
      bg_shifter_attrib_low: 0,
      bg_shifter_pattern_high: 0,
      bg_shifter_pattern_low: 0,
      frame_count: 0,
      status_register_read_this_tick: false,
      status_register_read_last_tick: false,
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

      if state.ppu.cycle == 338 || state.ppu.cycle == 340 {
        // superfluous reads of tile id at end of scanline
        state.ppu.bg_next_tile_id = state
          .ppu
          .get_ppu_mem(state, 0x2000 | (u16::from(state.ppu.vram_addr) & 0x0fff));
      }

      if state.ppu.scanline == -1 && state.ppu.cycle >= 280 && state.ppu.cycle < 305 {
        state.ppu.transfer_address_y();
      }
    }
  }

  fn draw_current_pixel(state: &mut Machine, pixbuf: &mut [u8; 245760]) {
    // Pixel is in the visible range of the CRT
    let mut color: [u8; 3] = [0, 0, 0];

    if state.ppu.mask.render_background() {
      color = state.ppu.get_current_pixel_bg_color(state);
    }

    state.ppu.set_pixel(
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
