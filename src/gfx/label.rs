use glyphon::{cosmic_text::Align, Attrs, FontSystem, Metrics, Shaping, TextArea, TextBounds};

pub struct Label {
  pub metrics: Metrics,
  pub attrs: Attrs<'static>,
  pub bounds: TextBounds,
  pub shaping: Shaping,
  pub buffer: glyphon::Buffer,
  pub align: Align,
}

impl Label {
  pub fn new(
    metrics: Metrics,
    bounds: TextBounds,
    attrs: Attrs<'static>,
    shaping: Shaping,
    buffer: glyphon::Buffer,
    align: Align,
  ) -> Self {
    Self {
      metrics,
      attrs,
      bounds,
      shaping,
      buffer,
      align,
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

  pub fn set_text(&mut self, font_system: &mut FontSystem, text: &str) {
    self
      .buffer
      .set_text(font_system, text, self.attrs, self.shaping);
    for line in &mut self.buffer.lines {
      line.set_align(Some(self.align));
    }

    self.buffer.shape_until_scroll(font_system);
  }
}
