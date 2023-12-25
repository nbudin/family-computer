use glyphon::{cosmic_text::Align, TextBounds};
use winit::{
  dpi::{PhysicalPosition, PhysicalSize},
  window::Window,
};

use super::{
  crt_screen::{CRTScreen, PIXEL_BUFFER_ASPECT, PIXEL_BUFFER_SIZE},
  node::{Node, RenderablePrepareData, RootNode},
  text_controller::{LabelID, TextController},
};

pub struct Layout {
  crt_screen: CRTScreen,
  text_controller: TextController,
  fps_label_id: LabelID,
}

impl Layout {
  pub fn new(data: &RenderablePrepareData) -> Self {
    let crt_screen = CRTScreen::new(
      &data.device,
      data.config.format,
      PhysicalPosition::new(0, 0),
      Self::calculate_crt_screen_size(&data.window),
    );
    let mut text_controller = TextController::new(&data.device, &data.queue, data.config.format);

    let fps_label_id = {
      text_controller.add_label(
        glyphon::Metrics {
          font_size: 2.0,
          line_height: 2.0,
        },
        Self::calculate_fps_label_bounds(&data.window),
        glyphon::Attrs::new()
          .family(glyphon::Family::Name("Pixel NES"))
          .color(glyphon::Color::rgb(255, 255, 255)),
        glyphon::Shaping::Basic,
        Align::Left,
      )
    };

    Self {
      crt_screen,
      fps_label_id,
      text_controller,
    }
  }

  fn calculate_crt_screen_size(window: &Window) -> PhysicalSize<u32> {
    let size_float: PhysicalSize<f32> = window.inner_size().cast();

    let aspect = (size_float.width / size_float.height) / PIXEL_BUFFER_ASPECT;
    let crt_screen_size = if aspect > 1.0 {
      PhysicalSize::new(size_float.width / aspect, size_float.height)
    } else {
      PhysicalSize::new(size_float.width, size_float.height * aspect)
    };

    crt_screen_size.cast()
  }

  fn calculate_fps_label_bounds(window: &Window) -> TextBounds {
    let crt_screen_size: PhysicalSize<i32> = Self::calculate_crt_screen_size(window).cast();

    TextBounds {
      top: 0,
      left: crt_screen_size.width + 10,
      right: window.inner_size().width as i32,
      bottom: window.inner_size().height as i32,
    }
  }

  pub fn update_pixbuf<F: FnOnce(&mut [u8; PIXEL_BUFFER_SIZE])>(&mut self, f: F) {
    f(&mut self.crt_screen.next_frame);
  }
}

impl Node for Layout {
  fn children(&self) -> Vec<Box<&dyn Node>> {
    vec![Box::new(&self.crt_screen), Box::new(&self.text_controller)]
  }

  fn children_mut(&mut self) -> Vec<Box<&mut dyn Node>> {
    vec![
      Box::new(&mut self.crt_screen),
      Box::new(&mut self.text_controller),
    ]
  }

  fn update(&mut self, delta_time: std::time::Duration) {
    self
      .text_controller
      .update_label(self.fps_label_id, |label, font_system| {
        label.set_text(
          font_system,
          format!("{:.02} FPS", 1.0 / delta_time.as_secs_f32()).as_str(),
        );
        label.buffer.shape_until_scroll(font_system);
      });
  }
}

impl RootNode for Layout {
  fn resize(&mut self, window: &Window) {
    self.crt_screen.size = Self::calculate_crt_screen_size(&window);
    self
      .text_controller
      .update_label(self.fps_label_id, |label, _font_system| {
        label.bounds = Self::calculate_fps_label_bounds(&window);
      });
  }
}
