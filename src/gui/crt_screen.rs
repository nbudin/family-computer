pub const PIXEL_BUFFER_WIDTH: u32 = 256;
pub const PIXEL_BUFFER_HEIGHT: u32 = 240;
pub const BYTES_PER_PIXEL: u32 = 4;
pub const PIXEL_BUFFER_SIZE: usize = 256 * 240 * 4;

pub struct CRTScreen {
  pub pixbuf: [u8; PIXEL_BUFFER_SIZE],
}

impl CRTScreen {
  pub fn new() -> Self {
    Self {
      pixbuf: [0; PIXEL_BUFFER_SIZE],
    }
  }

  pub fn image_handle(&self) -> iced::advanced::image::Handle {
    iced::advanced::image::Handle::from_pixels(PIXEL_BUFFER_WIDTH, PIXEL_BUFFER_HEIGHT, self.pixbuf)
  }
}
