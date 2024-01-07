use std::any::Any;

use dyn_clone::DynClone;

pub trait AudioChannel: DynClone + Send + Sync {
  fn tick(&mut self, sample_rate: f32) -> f32;
  fn handle_command(&mut self, command: Box<dyn Any + Send + Sync>);
}

impl Clone for Box<dyn AudioChannel> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(self.as_ref())
  }
}
