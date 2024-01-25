use std::ops::Rem;

use super::timing::APUTimerInstant;

pub trait APUChannel {
  fn playing(&self) -> bool;
  fn tick<T: Clone + Rem<u64, Output = T> + PartialEq<u64>>(&mut self, now: &APUTimerInstant<T>);
}
