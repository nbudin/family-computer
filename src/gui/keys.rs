use iced::keyboard::{self};
use iced_runtime::keyboard::Key;

use crate::{emulator::EmulatorState, nes::ControllerButton};

use super::EmulatorUIMessage;

pub fn key_code_to_controller_button(key_code: Key<&str>) -> Option<ControllerButton> {
  match key_code {
    Key::Character("s") => Some(ControllerButton::A),
    Key::Character("a") => Some(ControllerButton::B),
    Key::Named(keyboard::key::Named::Space) => Some(ControllerButton::Select),
    Key::Named(keyboard::key::Named::Enter) => Some(ControllerButton::Start),
    Key::Named(keyboard::key::Named::ArrowUp) => Some(ControllerButton::Up),
    Key::Named(keyboard::key::Named::ArrowDown) => Some(ControllerButton::Down),
    Key::Named(keyboard::key::Named::ArrowLeft) => Some(ControllerButton::Left),
    Key::Named(keyboard::key::Named::ArrowRight) => Some(ControllerButton::Right),
    _ => None,
  }
}

pub fn handle_key_event(event: iced::keyboard::Event) -> Option<EmulatorUIMessage> {
  match event {
    keyboard::Event::KeyPressed { key, .. } => {
      if let Some(button) = key_code_to_controller_button(key.as_ref()) {
        Some(EmulatorUIMessage::ControllerButtonChanged(button, true))
      } else {
        match key.as_ref() {
          Key::Character("r") => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::Run,
          )),
          Key::Character("p") => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::Pause,
          )),
          Key::Character("f") => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::RunUntilNextFrame,
          )),
          Key::Character("i") => Some(EmulatorUIMessage::EmulatorStateChangeRequested(
            EmulatorState::RunUntilNextInstruction,
          )),
          _ => None,
        }
      }
    }
    keyboard::Event::KeyReleased { key, .. } => key_code_to_controller_button(key.as_ref())
      .map(|button| EmulatorUIMessage::ControllerButtonChanged(button, false)),
    _ => None,
  }
}
