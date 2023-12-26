use bitfield_struct::bitfield;

use crate::{
  gfx::crt_screen::{BYTES_PER_PIXEL, PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_SIZE, PIXEL_BUFFER_WIDTH},
  machine::Machine,
  operand::Operand,
  palette::PALETTE,
};

#[derive(Debug)]
pub enum PPURegister {
  PPUCTRL,
  PPUMASK,
  PPUSTATUS,
  OAMADDR,
  OAMDATA,
  PPUSCROLL,
  PPUADDR,
  PPUDATA,
  OAMDMA,
}

#[derive(Debug)]
enum PPUAddressLatch {
  High,
  Low,
}

#[bitfield(u8)]
pub struct PPUStatusRegister {
  #[bits(5)]
  _unused: usize,
  sprite_overflow: bool,
  sprite_zero_hit: bool,
  vertical_blank: bool,
}

#[bitfield(u8)]
pub struct PPUMaskRegister {
  grayscale: bool,
  render_background_left: bool,
  render_sprites_left: bool,
  render_background: bool,
  render_sprites: bool,
  enhance_red: bool,
  enhance_green: bool,
  enhance_blue: bool,
}

#[bitfield(u8)]
pub struct PPUControlRegister {
  nametable_x: bool,
  nametable_y: bool,
  increment_mode: bool,
  pattern_sprite: bool,
  pattern_background: bool,
  sprite_size: bool,
  slave_mode: bool,
  enable_nmi: bool,
}

#[bitfield(u16)]
pub struct PPULoopyRegister {
  #[bits(5)]
  coarse_x: u8,
  #[bits(5)]
  coarse_y: u8,
  nametable_x: bool,
  nametable_y: bool,
  #[bits(3)]
  fine_y: u8,
  _unused: bool,
}

impl PPURegister {
  pub fn from_address(addr: u16) -> Self {
    match addr % 8 {
      0 => Self::PPUCTRL,
      1 => Self::PPUMASK,
      2 => Self::PPUSTATUS,
      3 => Self::OAMADDR,
      4 => Self::OAMDATA,
      5 => Self::PPUSCROLL,
      6 => Self::PPUADDR,
      7 => Self::PPUDATA,
      _ => panic!("This should never happen"),
    }
  }

  pub fn address(&self) -> Operand {
    match self {
      Self::PPUCTRL => Operand::Absolute(0x2000),
      Self::PPUMASK => Operand::Absolute(0x2001),
      Self::PPUSTATUS => Operand::Absolute(0x2002),
      Self::OAMADDR => Operand::Absolute(0x2003),
      Self::OAMDATA => Operand::Absolute(0x2004),
      Self::PPUSCROLL => Operand::Absolute(0x2005),
      Self::PPUADDR => Operand::Absolute(0x2006),
      Self::PPUDATA => Operand::Absolute(0x2007),
      Self::OAMDMA => Operand::Absolute(0x4014),
    }
  }
}

#[derive(Debug)]
pub struct PPU {
  pub x: u32,
  pub y: u32,
  status: PPUStatusRegister,
  mask: PPUMaskRegister,
  control: PPUControlRegister,
  vram_addr: PPULoopyRegister,
  tram_addr: PPULoopyRegister,
  fine_x: u8,
  data_bus: u8,
  address_latch: PPUAddressLatch,
  pub palette_ram: [u8; 32],
  name_tables: [[u8; 1024]; 2],
  pattern_tables: [[u8; 4096]; 2],
  bg_next_tile_id: u8,
  bg_next_tile_attrib: u8,
  bg_next_tile_low: u8,
  bg_next_tile_high: u8,
  bg_shifter_pattern_low: u16,
  bg_shifter_pattern_high: u16,
  bg_shifter_attrib_low: u16,
  bg_shifter_attrib_high: u16,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      mask: PPUMaskRegister::new(),
      control: PPUControlRegister::new(),
      status: PPUStatusRegister::new(),
      vram_addr: PPULoopyRegister::new(),
      tram_addr: PPULoopyRegister::new(),
      fine_x: 0,
      data_bus: 0,
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

  pub fn get_ppu_mem(&self, machine: &Machine, addr: u16) -> u8 {
    let cartridge = machine.cartridge.read().unwrap();

    match cartridge.get_ppu_mem(addr) {
      Some(value) => value,
      None => {
        if addr < 0x1fff {
          self.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff]
        } else if addr < 0x3f00 {
          let addr = addr & 0x0fff;

          match cartridge.get_mirroring() {
            crate::cartridge::CartridgeMirroring::HORIZONTAL => {
              if addr < 0x0400 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else if addr < 0x0800 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else if addr < 0x0c00 {
                self.name_tables[1][addr as usize & 0x03ff]
              } else {
                self.name_tables[1][addr as usize & 0x03ff]
              }
            }
            crate::cartridge::CartridgeMirroring::VERTICAL => {
              if addr < 0x0400 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else if addr < 0x0800 {
                self.name_tables[1][addr as usize & 0x03ff]
              } else if addr < 0x0c00 {
                self.name_tables[0][addr as usize & 0x03ff]
              } else {
                self.name_tables[1][addr as usize & 0x03ff]
              }
            }
          }
        } else {
          let addr = addr & 0x001f;
          let addr = match addr {
            0x0010 => 0x0000,
            0x0014 => 0x0004,
            0x0018 => 0x0008,
            0x001c => 0x000c,
            _ => addr,
          };
          self.palette_ram[addr as usize] & (if self.mask.grayscale() { 0x30 } else { 0x3f })
        }
      }
    }
  }

  pub fn set_ppu_mem(&mut self, machine: &Machine, addr: u16, value: u8) {
    let mut cartridge = (*machine.cartridge).write().unwrap();

    if cartridge.set_ppu_mem(addr, value) {
    } else {
      if addr < 0x2000 {
        self.pattern_tables[(addr as usize & 0x1000) >> 12][addr as usize & 0x0fff] = value;
      } else if addr < 0x3f00 {
        let addr = addr & 0x0fff;

        match cartridge.get_mirroring() {
          crate::cartridge::CartridgeMirroring::HORIZONTAL => {
            if addr < 0x0400 {
              self.name_tables[0][addr as usize & 0x03ff] = value;
            } else if addr < 0x0800 {
              self.name_tables[0][addr as usize & 0x03ff] = value;
            } else if addr < 0x0c00 {
              self.name_tables[1][addr as usize & 0x03ff] = value;
            } else {
              self.name_tables[1][addr as usize & 0x03ff] = value;
            }
          }
          crate::cartridge::CartridgeMirroring::VERTICAL => {
            if addr < 0x0400 {
              self.name_tables[0][addr as usize & 0x03ff] = value;
            } else if addr < 0x0800 {
              self.name_tables[1][addr as usize & 0x03ff] = value;
            } else if addr < 0x0c00 {
              self.name_tables[0][addr as usize & 0x03ff] = value;
            } else {
              self.name_tables[1][addr as usize & 0x03ff] = value;
            }
          }
        }
      } else {
        let addr = addr & 0x001f;
        let addr = match addr {
          0x0010 => 0x0000,
          0x0014 => 0x0004,
          0x0018 => 0x0008,
          0x001c => 0x000c,
          _ => addr,
        };
        self.palette_ram[addr as usize] = value;
      }
    }
  }

  pub fn read_bus(&mut self, machine: &Machine, register: PPURegister) -> u8 {
    let mut result: u8 = 0;

    match register {
      PPURegister::PPUSTATUS => {
        result = (u8::from(self.status) & 0b11100000) | (self.data_bus & 0b00011111);
        self.status.set_vertical_blank(false);
        self.address_latch = PPUAddressLatch::High;
      }
      PPURegister::PPUDATA => {
        result = self.data_bus;
        self.data_bus = self.get_ppu_mem(machine, self.vram_addr.into());

        if self.vram_addr.0 > 0x3f00 {
          // palette memory is read immediately
          result = self.data_bus;
        }

        self.vram_addr.0 += if self.control.increment_mode() { 32 } else { 1 };
      }
      _ => {}
    }

    result
  }

  pub fn write_bus(&mut self, machine: &Machine, register: PPURegister, value: u8) {
    match register {
      PPURegister::PPUCTRL => {
        self.control = value.into();
        self.tram_addr.set_nametable_x(self.control.nametable_x());
        self.tram_addr.set_nametable_y(self.control.nametable_y());
        self.data_bus = value;
      }
      PPURegister::PPUMASK => {
        self.mask = value.into();
        self.data_bus = value;
      }
      PPURegister::PPUSCROLL => match self.address_latch {
        PPUAddressLatch::High => {
          self.fine_x = value & 0x07;
          self.tram_addr.set_coarse_x(value >> 3);
          self.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          self.tram_addr.set_fine_y(value & 0x07);
          self.tram_addr.set_coarse_y(value >> 3);
          self.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUADDR => match self.address_latch {
        PPUAddressLatch::High => {
          self.tram_addr.0 = (self.tram_addr.0 & 0x00ff) | (u16::from(value) << 8);
          self.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          self.tram_addr.0 = (self.tram_addr.0 & 0xff00) | u16::from(value);
          self.vram_addr = self.tram_addr;
          self.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUDATA => {
        self.set_ppu_mem(machine, self.vram_addr.0, value);
        self.vram_addr.0 += if self.control.increment_mode() { 32 } else { 1 };
      }
      _ => {}
    }
  }

  pub fn get_bg_palette_color(&self, palette_index: usize, color_index: usize) -> u8 {
    self.palette_ram[1 + (palette_index * 4) + color_index]
  }

  pub fn get_sprite_palette_color(&self, palette_index: usize, color_index: usize) -> u8 {
    self.palette_ram[17 + (palette_index * 4) + color_index]
  }

  fn increment_scroll_x(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      if self.vram_addr.coarse_x() == 31 {
        self.vram_addr.set_coarse_x(0);
        self
          .vram_addr
          .set_nametable_x(!self.vram_addr.nametable_x());
      } else {
        self.vram_addr.set_coarse_x(self.vram_addr.coarse_x() + 1);
      }
    }
  }

  fn increment_scroll_y(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      if self.vram_addr.fine_y() < 7 {
        self.vram_addr.set_fine_y(self.vram_addr.fine_y() + 1);
      } else {
        self.vram_addr.set_fine_y(0);

        if self.vram_addr.coarse_y() == 29 {
          self.vram_addr.set_coarse_y(0);
          self
            .vram_addr
            .set_nametable_y(!self.vram_addr.nametable_y());
        } else if self.vram_addr.coarse_y() == 31 {
          self.vram_addr.set_coarse_y(0);
        } else {
          self.vram_addr.set_coarse_y(self.vram_addr.coarse_y() + 1);
        }
      }
    }
  }

  fn transfer_address_x(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
      self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
    }
  }

  fn transfer_address_y(&mut self) {
    if self.mask.render_background() || self.mask.render_sprites() {
      self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
      self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
    }
  }

  fn load_background_shifters(&mut self) {
    self.bg_shifter_pattern_low =
      (self.bg_shifter_pattern_low & 0xff00) | self.bg_next_tile_low as u16;
    self.bg_shifter_pattern_high =
      (self.bg_shifter_pattern_high & 0xff00) | self.bg_next_tile_high as u16;
    self.bg_shifter_attrib_low = (self.bg_shifter_attrib_low & 0xff00)
      | (if self.bg_next_tile_attrib & 0b01 > 0 {
        0xff
      } else {
        0
      });
    self.bg_shifter_attrib_high = (self.bg_shifter_attrib_high & 0xff00)
      | (if self.bg_next_tile_attrib & 0b10 > 0 {
        0xff
      } else {
        0
      });
  }

  fn update_shifters(&mut self) {
    if self.mask.render_background() {
      self.bg_shifter_pattern_high <<= 1;
      self.bg_shifter_pattern_low <<= 1;
      self.bg_shifter_attrib_high <<= 1;
      self.bg_shifter_attrib_low <<= 1;
    }
  }

  pub fn tick(&mut self, machine: &Machine, pixbuf: &mut [u8; PIXEL_BUFFER_SIZE]) {
    if self.x < PIXEL_BUFFER_WIDTH && self.y < PIXEL_BUFFER_HEIGHT {
      // Pixel is in the visible range of the CRT

      if self.mask.render_background() {
        let bit_mux = 0x8000 >> self.fine_x;

        let plane0_pixel: usize = if (self.bg_shifter_pattern_low & bit_mux) > 0 {
          1
        } else {
          0
        };
        let plane1_pixel: usize = if (self.bg_shifter_pattern_high & bit_mux) > 0 {
          0b10
        } else {
          0
        };
        let bg_pixel = plane1_pixel | plane0_pixel;

        let plane0_palette: usize = if (self.bg_shifter_attrib_low & bit_mux) > 0 {
          1
        } else {
          0
        };
        let plane1_palette: usize = if (self.bg_shifter_attrib_high & bit_mux) > 0 {
          0b10
        } else {
          0
        };
        let bg_palette = plane1_palette | plane0_palette;

        let color = PALETTE[self.get_bg_palette_color(bg_palette, bg_pixel) as usize % 64];

        // let name_table_offset = ((self.y as usize / 8) * 32) + (self.x as usize / 8);
        // let tile_index = self.name_tables[0][name_table_offset] as u16 + 256;
        // let color_index = self.get_tile_pixel(
        //   machine,
        //   tile_index,
        //   (self.x % 8) as u16,
        //   (self.y % 8) as u16,
        // );
        // // let palette_color = PALETTE[color_index as usize];
        // let color = PALETTE[self.get_sprite_palette_color(0, color_index.into()) as usize % 64];

        let offset = (self.x + (self.y * PIXEL_BUFFER_WIDTH)) * BYTES_PER_PIXEL;
        let pixel = pixbuf
          .get_mut((offset as usize)..((offset + BYTES_PER_PIXEL) as usize))
          .unwrap();
        pixel.copy_from_slice(&[color[0], color[1], color[2], 255]);
      }
    }

    if self.x == 1 && self.y == 261 {
      self.status.set_vertical_blank(false);
      self.status.set_sprite_zero_hit(false);
      self.status.set_sprite_overflow(false);
    }

    if (self.x >= 1 && self.x < 257) || (self.x >= 328 && self.x < 337) {
      self.update_shifters();

      match self.x % 8 {
        0 => {
          self.load_background_shifters();
          self.bg_next_tile_id = self.get_ppu_mem(machine, 0x2000 | self.vram_addr.0 & 0x0fff);
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
          )
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

      if self.x == 256 {
        self.increment_scroll_y();
      }

      if self.x == 257 {
        self.transfer_address_x();
      }

      if self.y == 261 && self.x >= 280 && self.x < 305 {
        self.transfer_address_y();
      }
    }

    // entering vblank
    if self.x == 0 && self.y == 240 {
      self.status.set_vertical_blank(true);
      if self.control.enable_nmi() {
        machine.nmi();
      }
    }

    if self.x < 341 {
      self.x += 1;
    } else if self.y < 262 {
      self.x = 0;
      self.y += 1;
    } else {
      self.x = 0;
      self.y = 0;
    }
  }
}
