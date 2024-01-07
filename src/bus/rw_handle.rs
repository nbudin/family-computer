use std::ops::Deref;

pub enum RwHandle<'a, T> {
  ReadOnly(&'a T),
  ReadWrite(&'a mut T),
}

impl<'a, T> RwHandle<'a, T> {
  pub fn try_mut(&mut self) -> Option<&mut T> {
    match self {
      RwHandle::ReadOnly(_) => None,
      RwHandle::ReadWrite(value) => Some(value),
    }
  }

  pub fn get_mut(&mut self) -> &mut T {
    self
      .try_mut()
      .expect("Tried to mutate a read-only RwHandle")
  }
}

impl<'a, T> Deref for RwHandle<'a, T> {
  type Target = T;

  fn deref(&self) -> &T {
    match self {
      RwHandle::ReadOnly(value) => value,
      RwHandle::ReadWrite(value) => value,
    }
  }
}
