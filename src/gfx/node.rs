use std::{fmt::Debug, time::Duration};
use winit::window::Window;

pub type BoxError = Box<dyn Debug>;

pub struct RenderablePrepareData<'a> {
  pub surface: &'a wgpu::Surface,
  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub config: &'a wgpu::SurfaceConfiguration,
  pub window: &'a Window,
}

#[allow(unused_variables)]
pub trait Node {
  fn prepare(&mut self, data: &RenderablePrepareData) -> Result<(), BoxError> {
    Ok(())
  }

  fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) -> Result<(), BoxError> {
    Ok(())
  }

  fn children(&self) -> Vec<Box<&dyn Node>> {
    vec![]
  }

  fn children_mut(&mut self) -> Vec<Box<&mut dyn Node>> {
    vec![]
  }

  fn prepare_recursive(&mut self, data: &RenderablePrepareData) -> Result<(), BoxError> {
    self.prepare(data)?;

    for child in self.children_mut() {
      child.prepare_recursive(data)?;
    }

    Ok(())
  }

  fn render_recursive<'pass>(
    &'pass self,
    pass: &mut wgpu::RenderPass<'pass>,
  ) -> Result<(), BoxError> {
    self.render(pass)?;

    for child in self.children() {
      child.render_recursive(pass)?;
    }

    Ok(())
  }

  fn update(&mut self, delta_time: Duration) {}
}

pub trait RootNode
where
  Self: Node,
{
  fn resize(&mut self, window: &Window);
}
