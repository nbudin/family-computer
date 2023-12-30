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

  pub fn read(&mut self) -> u8 {
    let result = if self.shift_register & 0x80 > 0 { 1 } else { 0 };
    self.shift_register <<= 1;
    result
  }

  pub fn read_readonly(&self) -> u8 {
    if self.shift_register & 0x80 > 0 {
      1
    } else {
      0
    }
  }

  pub fn poll(&mut self) {
    self.shift_register = self.state.into();
  }
}
