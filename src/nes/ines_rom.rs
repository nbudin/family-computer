use std::{
  fs::File,
  io::{Error, Read},
  path::Path,
};

use bitfield_struct::bitfield;

use crate::cartridge::CartridgeMirroring;

#[bitfield(u8)]
pub struct INESRomFlags6 {
  vertical_mirroring: bool,
  has_battery_ram: bool,
  has_trainer: bool,
  four_screen_vram: bool,
  #[bits(4)]
  mapper_low_nybble: u8,
}

#[bitfield(u8)]
pub struct INESRomFlags7 {
  vs_unisystem: bool,
  playchoice_10: bool,
  #[bits(2)]
  nes20_format: u8,
  #[bits(4)]
  mapper_high_nybble: u8,
}

#[derive(Debug, Clone)]
pub struct INESRom {
  pub prg_data: Vec<u8>,
  pub chr_data: Vec<u8>,
  pub trainer_data: Option<Vec<u8>>,
  pub has_battery_ram: bool,
  pub vertical_mirroring: bool,
  pub mapper_id: u16,
  pub playchoice_10: bool,
  pub vs_unisystem: bool,
  pub uses_chr_ram: bool,
  pub four_screen_vram: bool,
}

impl INESRom {
  pub fn from_file(path: &Path) -> Result<Self, Error> {
    let mut file = File::open(path)?;
    Self::from_reader(&mut file)
  }

  pub fn from_reader<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Error> {
    let mut header: [u8; 16] = [0; 16];
    reader.read_exact(&mut header)?;

    let prg_size: usize = usize::from(header[4]) * 16 * 1024;
    let chr_size: usize = usize::from(header[5]) * 8 * 1024;
    let uses_chr_ram = chr_size == 0;

    let flags6 = INESRomFlags6::from(header[6]);
    let flags7 = INESRomFlags7::from(header[7]);

    let mapper_id = (flags7.mapper_high_nybble() << 4) | flags6.mapper_low_nybble();

    let mut trainer_data: Option<Vec<u8>> = None;

    if flags6.has_trainer() {
      let mut trainer_buf: [u8; 512] = [0; 512];
      reader.read_exact(&mut trainer_buf)?;
      trainer_data = Some(trainer_buf.into());
    }

    let mut prg_buf = Vec::with_capacity(prg_size);
    reader
      .take(prg_size.try_into().unwrap())
      .read_to_end(&mut prg_buf)?;

    let mut chr_buf = Vec::with_capacity(chr_size);
    reader
      .take(chr_size.try_into().unwrap())
      .read_to_end(&mut chr_buf)?;

    Ok(Self {
      chr_data: chr_buf,
      prg_data: prg_buf,
      trainer_data,
      has_battery_ram: flags6.has_battery_ram(),
      vertical_mirroring: flags6.vertical_mirroring(),
      playchoice_10: flags7.playchoice_10(),
      vs_unisystem: flags7.vs_unisystem(),
      mapper_id: u16::from(mapper_id),
      uses_chr_ram,
      four_screen_vram: flags6.four_screen_vram(),
    })
  }

  pub fn initial_mirroring(&self) -> CartridgeMirroring {
    if self.four_screen_vram {
      CartridgeMirroring::FourScreen
    } else if self.vertical_mirroring {
      CartridgeMirroring::Vertical
    } else {
      CartridgeMirroring::Horizontal
    }
  }
}
