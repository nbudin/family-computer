use bitfield_struct::bitfield;

#[bitfield(u8)]
pub struct ControllerState {
  pub right: bool,
  pub left: bool,
  pub down: bool,
  pub up: bool,
  pub start: bool,
  pub select: bool,
  pub b: bool,
  pub a: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Controller {
  state: ControllerState,
  shift_register: u8,
}

impl Controller {
  pub fn new() -> Self {
    Self {
      state: ControllerState::new(),
      shift_register: 0,
    }
  }

  pub fn update<F: FnOnce(&mut ControllerState) -> ()>(&mut self, f: F) {
    f(&mut self.state);
  }

  pub fn read(&mut self) -> u8 {
    let result = if self.shift_register & 0x80 > 0 { 1 } else { 0 };
    self.shift_register <<= 1;
    result
  }

  pub fn write(&mut self) {
    self.shift_register = self.state.into();
  }
}
