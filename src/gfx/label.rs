use glyphon::{Attrs, Metrics, Shaping, TextArea, TextBounds};

#[derive(Debug)]
pub struct Label {
  pub metrics: Metrics,
  pub attrs: Attrs<'static>,
  pub bounds: TextBounds,
  pub shaping: Shaping,
}

impl Label {
  pub fn new(
    metrics: Metrics,
    bounds: TextBounds,
    attrs: Attrs<'static>,
    shaping: Shaping,
  ) -> Self {
    Self {
      metrics,
      attrs,
      bounds,
      shaping,
    }
  }

  pub fn text_area<'a>(&'a self, buffer: &'a glyphon::Buffer) -> TextArea<'_> {
    TextArea {
      left: self.bounds.left as f32,
      top: self.bounds.top as f32,
      scale: 16.0,
      bounds: self.bounds,
      default_color: glyphon::Color::rgb(255, 255, 255),
      buffer,
    }
  }
}
