use glyphon::{
  Attrs, FontSystem, Metrics, RenderError, Resolution, Shaping, SwashCache, TextAtlas, TextBounds,
  TextRenderer,
};
use wgpu::MultisampleState;

use super::label::Label;

pub type LabelID = usize;

pub struct TextController {
  labels: Vec<Label>,
  buffers: Vec<glyphon::Buffer>,
  text_renderer: TextRenderer,
  atlas: TextAtlas,
  swash_cache: SwashCache,
  font_system: FontSystem,
}

impl TextController {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_format: wgpu::TextureFormat,
  ) -> Self {
    let mut atlas = TextAtlas::new(device, queue, texture_format);
    let text_renderer = TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
    let swash_cache = SwashCache::new();
    let mut font_system = FontSystem::new();

    font_system
      .db_mut()
      .load_font_data(include_bytes!("Pixel_NES.otf").to_vec());

    Self {
      labels: vec![],
      buffers: vec![],
      text_renderer,
      atlas,
      swash_cache,
      font_system,
    }
  }

  pub fn add_label(
    &mut self,
    metrics: Metrics,
    bounds: TextBounds,
    attrs: Attrs<'static>,
    shaping: Shaping,
  ) -> LabelID {
    let id = self.labels.len();

    self
      .labels
      .push(Label::new(metrics, bounds, attrs, shaping));

    let mut text_buffer = glyphon::Buffer::new(&mut self.font_system, metrics);

    text_buffer.set_size(
      &mut self.font_system,
      (bounds.right - bounds.left) as f32,
      (bounds.bottom - bounds.top) as f32,
    );

    self.buffers.push(text_buffer);

    id
  }

  pub fn set_label_text(&mut self, label_id: LabelID, text: &str) {
    self.buffers[label_id].set_text(
      &mut self.font_system,
      text,
      self.labels[label_id].attrs,
      self.labels[label_id].shaping,
    );

    self.buffers[label_id].shape_until_scroll(&mut self.font_system);
  }

  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, resolution: Resolution) {
    self
      .text_renderer
      .prepare(
        device,
        queue,
        &mut self.font_system,
        &mut self.atlas,
        resolution,
        self
          .labels
          .iter()
          .zip(self.buffers.iter())
          .map(|(label, buffer)| label.text_area(buffer)),
        &mut self.swash_cache,
      )
      .unwrap();
  }

  pub fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) -> Result<(), RenderError> {
    self.text_renderer.render(&self.atlas, pass)
  }
}
