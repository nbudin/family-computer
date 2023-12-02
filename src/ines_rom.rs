use std::{
  fs::File,
  io::{Error, Read},
  path::Path,
};

#[derive(Debug)]
pub struct INESRom {
  pub prg_data: Vec<u8>,
  pub chr_data: Vec<u8>,
  pub trainer_data: Option<Vec<u8>>,
}

impl INESRom {
  pub fn from_file(path: &Path) -> Result<Self, Error> {
    let mut file = File::open(path)?;
    Self::from_reader(&mut file)
  }

  pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
    let mut header: [u8; 16] = [0; 16];
    reader.read_exact(&mut header)?;

    let prg_size: usize = usize::from(header[4]) * 16 * 1024;
    let chr_size: usize = usize::from(header[5]) * 8 * 1024;
    let flags6 = header[6];
    let has_trainer = (flags6 & 0b100) > 0;

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
    })
  }
}
