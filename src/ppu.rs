use crate::{
  gfx::{
    crt_screen::{BYTES_PER_PIXEL, PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_WIDTH},
    gfx_state::GfxState,
  },
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
      7 => Self::OAMDMA,
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
  nmi_enable: bool,
  master_slave: bool,
  sprite_height: bool,
  background_tile_select: bool,
  increment_mode: bool,
  sprite_overflow: bool,
  even_frame: bool,
  sprite0_hit: bool,
  nametable_select: u8,
  data_bus: u8,
  pub palette_ram: [u8; 32],
}

impl PPU {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      nmi_enable: false,
      master_slave: false,
      sprite_height: false,
      background_tile_select: false,
      increment_mode: false,
      sprite0_hit: false,
      sprite_overflow: false,
      even_frame: false,
      nametable_select: 0,
      data_bus: 0,
      palette_ram: [0; 32],
    }
  }

  pub fn get_ppu_mem(&self, machine: &Machine, addr: u16) -> u8 {
    if addr < 0x3f00 {
      machine.cartridge.read().unwrap().get_ppu_mem(addr)
    } else {
      self.palette_ram[usize::from(addr) % 32]
    }
  }

  pub fn set_ppu_mem(&mut self, machine: &Machine, addr: u16, value: u8) {
    if addr < 0x3f00 {
      let mut cartridge = (*machine.cartridge).write().unwrap();
      cartridge.set_ppu_mem(addr, value)
    } else {
      self.palette_ram[usize::from(addr) % 32] = value;
    }
  }

  pub fn read_bus(&mut self, register: PPURegister) -> u8 {
    match register {
      PPURegister::PPUSTATUS => {
        self.data_bus = (self.data_bus & 0b00011111)
          + (if self.y > 239 { 1 << 7 } else { 0 })
          + (if self.sprite0_hit { 1 << 6 } else { 0 })
          + (if self.sprite_overflow { 1 << 5 } else { 0 })
      }
      _ => {}
    }

    self.data_bus
  }

  pub fn write_bus(&mut self, register: PPURegister, value: u8) {
    match register {
      PPURegister::PPUCTRL => {
        self.nmi_enable = (value & (1 << 7)) > 0;
        self.master_slave = (value & (1 << 6)) > 0;
        self.sprite_height = (value & (1 << 5)) > 0;
        self.background_tile_select = (value & (1 << 4)) > 0;
        // TODO low 4 bits

        self.data_bus = value;
      }
      _ => {}
    }
  }

  pub fn get_tile_pixel(&self, machine: &Machine, tile_index: u16, x: u16, y: u16) -> u8 {
    let tile_offset = tile_index * 16;
    let plane1_row = self.get_ppu_mem(machine, tile_offset + y);
    let plane2_row = self.get_ppu_mem(machine, tile_offset + y + 8);

    let plane1_bit = (plane1_row >> (7 - x)) & 1;
    let plane2_bit = (plane2_row >> (7 - x)) & 1;

    (plane2_bit << 1) + plane1_bit
  }

  pub fn get_bg_palette_color(&self, palette_index: usize, color_index: usize) -> u8 {
    self.palette_ram[1 + (palette_index * 4) + color_index]
  }

  pub fn get_sprite_palette_color(&self, palette_index: usize, color_index: usize) -> u8 {
    self.palette_ram[17 + (palette_index * 4) + color_index]
  }

  pub fn tick(&mut self, machine: &Machine, gfx_state: &mut GfxState) {
    if self.x < PIXEL_BUFFER_WIDTH && self.y < PIXEL_BUFFER_HEIGHT {
      // Pixel is in the visible range of the CRT
      let offset = (self.x + (self.y * PIXEL_BUFFER_WIDTH)) * BYTES_PER_PIXEL;
      let pixel = gfx_state
        .get_pixbuf_mut()
        .get_mut((offset as usize)..((offset + BYTES_PER_PIXEL) as usize))
        .unwrap();
      let color_index = self.get_tile_pixel(
        machine,
        0,
        (self.x / (PIXEL_BUFFER_WIDTH / 8)) as u16,
        (self.y / (PIXEL_BUFFER_HEIGHT / 8)) as u16,
      );
      let palette_color = PALETTE[color_index as usize];
      // let palette_color = PALETTE[self.get_bg_palette_color(0, color_index.into()) as usize];

      pixel.copy_from_slice(&[palette_color[0], palette_color[1], palette_color[2], 255]);
    }

    if self.x < 341 {
      self.x += 1;
    } else if self.y < 262 {
      self.x = 0;
      self.y += 1;
      if self.y == 240 && self.nmi_enable {
        machine.nmi();
      }
    } else {
      self.x = 0;
      self.y = 0;
    }
  }
}
