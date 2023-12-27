use crate::{
  gfx::crt_screen::{PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_SIZE, PIXEL_BUFFER_WIDTH},
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
    }
  }

  pub fn tick(mut self, machine: &mut Machine, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) -> Self {
    if self.scanline >= -1 && self.scanline < 240 {
      if self.scanline == 0 && self.cycle == 0 {
        // Odd frame cycle skip
        self.cycle = 1;
      }

      if self.scanline == -1 && self.cycle == 1 {
        self.status.set_vertical_blank(false);
        self.status.set_sprite_zero_hit(false);
        self.status.set_sprite_overflow(false);
      }

      if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
        self.update_shifters();

        match (self.cycle - 1) % 8 {
          0 => {
            self.load_background_shifters();
            self.bg_next_tile_id =
              self.get_ppu_mem(machine, 0x2000 | u16::from(self.vram_addr) & 0x0fff);
          }
          2 => {
            self.bg_next_tile_attrib = self.get_ppu_mem(
              machine,
              0x23c0
                | (if self.vram_addr.nametable_y() {
                  1 << 11
                } else {
                  0
                })
                | (if self.vram_addr.nametable_x() {
                  1 << 10
                } else {
                  0
                })
                | ((self.vram_addr.coarse_y() as u16 >> 2) << 3)
                | (self.vram_addr.coarse_x() as u16 >> 2),
            );

            if self.vram_addr.coarse_y() & 0x02 > 0 {
              self.bg_next_tile_attrib >>= 4;
            }
            if self.vram_addr.coarse_x() & 0x02 > 0 {
              self.bg_next_tile_attrib >>= 2;
            }
            self.bg_next_tile_attrib &= 0x03;
          }
          4 => {
            self.bg_next_tile_low = self.get_ppu_mem(
              machine,
              (if self.control.pattern_background() {
                1 << 12
              } else {
                0
              }) + ((self.bg_next_tile_id as u16) << 4)
                + (self.vram_addr.fine_y() as u16),
            )
          }
          6 => {
            self.bg_next_tile_high = self.get_ppu_mem(
              machine,
              (if self.control.pattern_background() {
                1 << 12
              } else {
                0
              }) + ((self.bg_next_tile_id as u16) << 4)
                + (self.vram_addr.fine_y() as u16)
                + 8,
            )
          }
          7 => {
            self.increment_scroll_x();
          }
          _ => {}
        }

        if self.cycle == 256 {
          self.increment_scroll_y();
        }

        if self.cycle == 257 {
          self.load_background_shifters();
          self.transfer_address_x();
        }

        if self.cycle == 338 || self.cycle == 340 {
          // superfluous reads of tile id at end of scanline
          self.bg_next_tile_id =
            self.get_ppu_mem(machine, 0x2000 | (u16::from(self.vram_addr) & 0x0fff));
        }

        if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
          self.transfer_address_y();
        }
      }
    }

    if self.scanline == 240 {
      // post render scanline - do nothing
    }

    if self.scanline >= 241 && self.scanline < 261 {
      if self.scanline == 241 && self.cycle == 1 {
        // entering vblank
        self.status.set_vertical_blank(true);
        if self.control.enable_nmi() {
          machine.nmi();
        }
      }
    }

    if self.cycle >= 1
      && self.scanline >= 0
      && self.cycle <= PIXEL_BUFFER_WIDTH as i32
      && self.scanline < PIXEL_BUFFER_HEIGHT as i32
    {
      // Pixel is in the visible range of the CRT
      let mut color: [u8; 3] = [0, 0, 0];

      if self.mask.render_background() {
        color = self.get_current_pixel_bg_color(machine);
      }

      self.set_pixel(
        pixbuf,
        color,
        u32::try_from(self.cycle - 1).unwrap(),
        u32::try_from(self.scanline).unwrap(),
      );
    }

    self.cycle += 1;
    if self.cycle >= 341 {
      self.cycle = 0;
      self.scanline += 1;
      if self.scanline >= 261 {
        self.scanline = -1;
      }
    }

    self
  }
}
