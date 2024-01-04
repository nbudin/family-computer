use iced::keyboard::{self, KeyCode};

use crate::{controller::ControllerButton, emulator::EmulatorState};

use super::EmulatorUIMessage;

pub fn key_code_to_controller_button(key_code: KeyCode) -> Option<ControllerButton> {
  match key_code {
    KeyCode::S => Some(ControllerButton::A),
    KeyCode::A => Some(ControllerButton::B),
    KeyCode::Space => Some(ControllerButton::Select),
    KeyCode::Enter => Some(ControllerButton::Start),
    KeyCode::Up => Some(ControllerButton::Up),
    KeyCode::Down => Some(ControllerButton::Down),
    KeyCode::Left => Some(ControllerButton::Left),
    KeyCode::Right => Some(ControllerButton::Right),
    _ => None,
  }
}

pub fn handle_key_event(event: iced::keyboard::Event) -> Option<EmulatorUIMessage> {
  match event {
    keyboard::Event::KeyPressed {
      key_code,
      modifiers: _,
    } => {
      if let Some(button) = key_code_to_controller_button(key_code) {
        Some(EmulatorUIMessage::ControllerButtonChanged(button, true))
      } else {
        match key_code {
          KeyCode::R => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::Run,
          )),
          KeyCode::P => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::Pause,
          )),
          KeyCode::F => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::RunUntilNextFrame,
          )),
          KeyCode::I => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::RunUntilNextInstruction,
          )),
          _ => None,
        }
      }
    }
    keyboard::Event::KeyReleased {
      key_code,
      modifiers: _,
    } => key_code_to_controller_button(key_code)
      .map(|button| EmulatorUIMessage::ControllerButtonChanged(button, false)),
    _ => None,
  }
}
