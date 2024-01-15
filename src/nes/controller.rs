use bitfield_struct::bitfield;

use crate::bus::Bus;

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

#[derive(Debug, Clone, Copy)]
pub enum ControllerButton {
  Right,
  Left,
  Down,
  Up,
  Start,
  Select,
  B,
  A,
}

#[derive(Clone, Debug)]
pub struct Controller {
  pub state: ControllerState,
  shift_register: u8,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
  pub fn new() -> Self {
    Self {
      state: ControllerState::new(),
      shift_register: 0,
    }
  }

  pub fn set_button_state(&mut self, button: ControllerButton, pressed: bool) {
    match button {
      ControllerButton::Right => self.state.set_right(pressed),
      ControllerButton::Left => self.state.set_left(pressed),
      ControllerButton::Down => self.state.set_down(pressed),
      ControllerButton::Up => self.state.set_up(pressed),
      ControllerButton::Start => self.state.set_start(pressed),
      ControllerButton::Select => self.state.set_select(pressed),
      ControllerButton::B => self.state.set_b(pressed),
      ControllerButton::A => self.state.set_a(pressed),
    }
  }

  pub fn poll(&mut self) {
    // it doesn't matter what you write to the controller, it always shifts the shift register
    self.write((), 0);
  }
}

impl Bus<()> for Controller {
  fn try_read_readonly(&self, _addr: ()) -> Option<u8> {
    if self.shift_register & 0x80 > 0 {
      Some(1)
    } else {
      Some(0)
    }
  }

  fn read_side_effects(&mut self, _addr: ()) {
    self.shift_register <<= 1;
  }

  fn write(&mut self, _addr: (), _value: u8) {
    self.shift_register = self.state.into();
  }
}
