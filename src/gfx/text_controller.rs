use glyphon::{
  cosmic_text::Align, Attrs, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextAtlas,
  TextBounds, TextRenderer,
};
use wgpu::MultisampleState;

use super::{
  label::Label,
  node::{BoxError, Node, RenderablePrepareData},
};

pub type LabelID = usize;

pub struct TextController {
  labels: Vec<Label>,
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
    align: Align,
  ) -> LabelID {
    let id = self.labels.len();

    let mut text_buffer = glyphon::Buffer::new(&mut self.font_system, metrics);

    text_buffer.set_size(
      &mut self.font_system,
      (bounds.right - bounds.left) as f32,
      (bounds.bottom - bounds.top) as f32,
    );

    self.labels.push(Label::new(
      metrics,
      bounds,
      attrs,
      shaping,
      text_buffer,
      align,
    ));

    id
  }

  pub fn update_label<F: FnMut(&mut Label, &mut FontSystem) -> ()>(
    &mut self,
    label_id: LabelID,
    mut f: F,
  ) {
    f(&mut self.labels[label_id], &mut self.font_system)
  }
}

impl Node for TextController {
  fn prepare(&mut self, data: &RenderablePrepareData) -> Result<(), BoxError> {
    let size = data.window.inner_size();

    self
      .text_renderer
      .prepare(
        &data.device,
        &data.queue,
        &mut self.font_system,
        &mut self.atlas,
        Resolution {
          width: size.width,
          height: size.height,
        },
        self
          .labels
          .iter()
          .map(|label| label.text_area(&label.buffer)),
        &mut self.swash_cache,
      )
      .map_err(|err| Box::new(err) as BoxError)
  }

  fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) -> Result<(), BoxError> {
    self
      .text_renderer
      .render(&self.atlas, pass)
      .map_err(|err| Box::new(err) as BoxError)
  }
}
