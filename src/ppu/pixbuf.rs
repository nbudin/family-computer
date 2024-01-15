pub const PIXEL_BUFFER_WIDTH: u32 = 256;
pub const PIXEL_BUFFER_HEIGHT: u32 = 240;
pub const BYTES_PER_PIXEL: u32 = 4;
pub const PIXEL_BUFFER_SIZE: usize = 256 * 240 * 4;

pub struct Pixbuf {
  pub data: [u8; PIXEL_BUFFER_SIZE],
}

impl Default for Pixbuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Pixbuf {
  pub fn new() -> Self {
    Self {
      data: [0; PIXEL_BUFFER_SIZE],
    }
  }

  pub fn set_pixel(&mut self, color: [u8; 3], x: u32, y: u32) {
    let offset = (x + (y * PIXEL_BUFFER_WIDTH)) * BYTES_PER_PIXEL;
    let pixel = self
      .data
      .get_mut((offset as usize)..((offset + BYTES_PER_PIXEL) as usize))
      .unwrap();
    pixel.copy_from_slice(&[color[0], color[1], color[2], 255]);
  }
}
