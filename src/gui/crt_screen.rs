use std::sync::{Arc, RwLock};

use crate::ppu::{Pixbuf, PIXEL_BUFFER_HEIGHT, PIXEL_BUFFER_WIDTH};

pub struct CRTScreen {
  pub pixbuf: Arc<RwLock<Pixbuf>>,
}

impl CRTScreen {
  pub fn new() -> Self {
    Self {
      pixbuf: Arc::new(RwLock::new(Pixbuf::new())),
    }
  }

  pub fn image_handle(&self) -> iced::advanced::image::Handle {
    iced::advanced::image::Handle::from_pixels(
      PIXEL_BUFFER_WIDTH,
      PIXEL_BUFFER_HEIGHT,
      self.pixbuf.read().unwrap().data,
    )
  }
}
