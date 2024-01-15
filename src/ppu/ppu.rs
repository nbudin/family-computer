use crate::{bus::Bus, nes::NES};

use super::{
  registers::{PPUControlRegister, PPULoopyRegister, PPUMaskRegister, PPUStatusRegister},
  sprites::PPUOAMEntry,
  ActiveSprite, Pixbuf,
};

#[derive(Debug, Clone, Copy)]
pub enum PPUAddressLatch {
  High = 0,
  Low = 1,
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
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

impl Default for PPU {
  fn default() -> Self {
    Self::new()
  }
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

  fn start_frame(nes: &mut NES) {
    nes.ppu.status.set_vertical_blank(false);
    nes.ppu.status.set_sprite_zero_hit(false);
    nes.ppu.status.set_sprite_overflow(false);

    nes.ppu.sprite_shifter_pattern_low = [0; 8];
    nes.ppu.sprite_shifter_pattern_high = [0; 8];
  }

  fn update_registers_on_renderable_scanline(nes: &mut NES) {
    if (nes.ppu.cycle >= 1 && nes.ppu.cycle < 258) || (nes.ppu.cycle >= 321 && nes.ppu.cycle < 338)
    {
      nes.ppu.update_shifters();

      PPU::update_bg_registers(nes);
    }

    if nes.ppu.cycle == 256 {
      nes.ppu.increment_scroll_y();
    }

    if nes.ppu.cycle == 257 {
      nes.ppu.load_background_shifters();
      nes.ppu.transfer_address_x();
    }

    if nes.ppu.cycle == 338 || nes.ppu.cycle == 340 {
      // superfluous reads of tile id at end of scanline
      let addr = 0x2000 | (u16::from(nes.ppu.vram_addr) & 0x0fff);
      let next_tile_id = nes.ppu_memory_mut().read(addr);
      nes.ppu.bg_next_tile_id = next_tile_id;
    }

    if nes.ppu.scanline == -1 && nes.ppu.cycle >= 280 && nes.ppu.cycle < 305 {
      nes.ppu.transfer_address_y();
    }

    // Foreground rendering =========================================================
    if nes.ppu.cycle == 257 && nes.ppu.scanline >= 0 {
      PPU::evaluate_scanline_sprites(nes);
    }

    if nes.ppu.cycle == 340 {
      for sprite_index in 0..nes.ppu.sprite_scanline.len() {
        PPU::load_sprite_data_for_next_scanline(nes, sprite_index);
      }
    }
  }

  fn increment_cycle_and_scanline(nes: &mut NES) {
    nes.ppu.cycle += 1;

    if nes.ppu.cycle >= 341 {
      nes.ppu.cycle = 0;
      nes.ppu.scanline += 1;

      if nes.ppu.scanline >= 261 {
        nes.ppu.scanline = -1;
        nes.ppu.frame_count += 1;
      }
    }
  }

  pub fn tick(nes: &mut NES, pixbuf: &mut Pixbuf) -> bool {
    let mut trigger_nmi = false;
    nes.ppu.status_register_read_last_tick = nes.ppu.status_register_read_this_tick;
    nes.ppu.status_register_read_this_tick = false;

    if nes.ppu.scanline >= -1 && nes.ppu.scanline < 240 {
      if nes.ppu.scanline == 0 && nes.ppu.cycle == 0 && nes.ppu.frame_count % 2 == 1 {
        // Odd frame cycle skip
        if nes.ppu.mask.render_background() || nes.ppu.mask.render_sprites() {
          nes.ppu.cycle = 1;
        }
      }

      if nes.ppu.scanline == -1 && nes.ppu.cycle == 1 {
        PPU::start_frame(nes);
      }

      PPU::update_registers_on_renderable_scanline(nes);
    }

    if nes.ppu.scanline == 241 && nes.ppu.cycle == 1 {
      // emulate a race condition in the PPU: reading the status register suppresses vblank next tick and nmi this tick
      if !nes.ppu.status_register_read_last_tick {
        nes.ppu.status.set_vertical_blank(true);
      }

      if nes.ppu.control.enable_nmi() && !nes.ppu.status_register_read_this_tick {
        trigger_nmi = true;
      }
    }

    PPU::draw_current_pixel(nes, pixbuf);
    PPU::increment_cycle_and_scanline(nes);

    trigger_nmi
  }
}
