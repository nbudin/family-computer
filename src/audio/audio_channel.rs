use std::{any::Any, fmt::Debug, time::Duration};

use dyn_clone::DynClone;

pub trait AudioChannel: DynClone + Debug + Send + Sync {
  fn get_next_sample(&mut self, sample_rate: f32, timestamp: Duration) -> f32;
  fn handle_command(&mut self, command: Box<dyn Any + Send + Sync>);
  fn mix_amplitude(&self) -> f32;
}

impl Clone for Box<dyn AudioChannel> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(self.as_ref())
  }
}
