use std::{
  fs::File,
  io::{Error, Read},
  path::Path,
};

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

    let flags6 = header[6];
    let has_trainer = (flags6 & 0b100) > 0;
    let has_battery_ram = (flags6 & 0b10) > 0;
    let vertical_mirroring = (flags6 & 0b1) > 0;
    let mapper_low_nybble = flags6 >> 4;

    let flags7 = header[7];
    let _nes20_format = ((flags7 >> 2) & 0b11) == 2;
    let playchoice_10 = (flags7 & 0b10) > 0;
    let vs_unisystem = (flags7 & 0b1) > 0;
    let mapper_high_nybble = flags7 >> 4;

    let mapper_id = (mapper_high_nybble << 4) + mapper_low_nybble;

    let mut trainer_data: Option<Vec<u8>> = None;

    if has_trainer {
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
      chr_data: chr_buf.into(),
      prg_data: prg_buf.into(),
      trainer_data,
      has_battery_ram,
      vertical_mirroring,
      playchoice_10,
      vs_unisystem,
      mapper_id: u16::from(mapper_id),
      uses_chr_ram,
    })
  }
}
