use std::{
  f32::consts::PI,
  time::{Duration, Instant},
};

use iced::{
  executor,
  keyboard::{self, KeyCode},
  theme::Palette,
  widget::{column, image, row, text},
  Application, Color, Command, Font, Length, Subscription, Theme,
};
use strum::IntoStaticStr;

use crate::{controller::ControllerButton, machine::Machine};

use super::CRTScreen;

const PIXEL_NES_FONT: Font = Font::with_name("Pixel NES");

#[derive(Debug, Clone, Copy, IntoStaticStr)]
pub enum EmulatorState {
  Run,
  Pause,
  RunUntilNextFrame,
  RunUntilNextInstruction,
}

pub struct EmulatorUIFlags {
  machine: Machine,
}

impl EmulatorUIFlags {
  pub fn new(machine: Machine) -> Self {
    Self { machine }
  }
}

fn key_code_to_controller_button(key_code: KeyCode) -> Option<ControllerButton> {
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

pub struct EmulatorUI {
  emulator_state: EmulatorState,
  machine: Machine,
  crt_screen: CRTScreen,
  last_tick: Instant,
  last_tick_duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
  ControllerButtonChanged(ControllerButton, bool),
  EmulatorStateChangeRequested(EmulatorState),
  Tick,
  FontLoaded(Result<(), iced::font::Error>),
}

impl Application for EmulatorUI {
  type Executor = executor::Default;
  type Message = Message;
  type Flags = EmulatorUIFlags;
  type Theme = Theme;

  fn new(flags: EmulatorUIFlags) -> (EmulatorUI, Command<Self::Message>) {
    (
      EmulatorUI {
        emulator_state: EmulatorState::Pause,
        machine: flags.machine,
        crt_screen: CRTScreen::new(),
        last_tick: Instant::now(),
        last_tick_duration: Duration::from_millis(1000),
      },
      iced::font::load(include_bytes!("./Pixel_NES.otf").as_slice()).map(Message::FontLoaded),
    )
  }

  fn theme(&self) -> Self::Theme {
    Theme::custom(Palette {
      background: Color::from_rgb(0.1, 0.2, 0.3),
      text: Color::WHITE,
      primary: Color::from_rgb(0.0, 0.0, 1.0),
      success: Color::from_rgb(0.0, 1.0, 0.0),
      danger: Color::from_rgb(1.0, 0.0, 0.0),
    })
  }

  fn title(&self) -> String {
    String::from("Family Computer")
  }

  fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
      Message::FontLoaded(_) => Command::none(),
      Message::ControllerButtonChanged(button, pressed) => {
        self.machine.controllers[0].set_button_state(button, pressed);
        Command::none()
      }
      Message::EmulatorStateChangeRequested(new_state) => {
        self.emulator_state = new_state;
        Command::none()
      }
      Message::Tick => {
        let now = Instant::now();
        self.last_tick_duration = now - self.last_tick;
        self.last_tick = now;

        match self.emulator_state {
          EmulatorState::Run => {
            self.machine.execute_frame(&mut self.crt_screen.pixbuf);
          }
          EmulatorState::Pause => {}
          EmulatorState::RunUntilNextFrame => {
            self.machine.execute_frame(&mut self.crt_screen.pixbuf);
            self.emulator_state = EmulatorState::Pause;
          }
          EmulatorState::RunUntilNextInstruction => {
            let start_cycles = self.machine.cpu_cycle_count;
            loop {
              self.machine.tick(&mut self.crt_screen.pixbuf);

              if self.machine.cpu_cycle_count > start_cycles {
                break;
              }
            }

            self.emulator_state = EmulatorState::Pause;
          }
        }

        Command::none()
      }
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    iced::Subscription::batch([
      iced::subscription::events_with(|event, _status| match event {
        iced::Event::Keyboard(event) => match event {
          keyboard::Event::KeyPressed {
            key_code,
            modifiers: _,
          } => {
            if let Some(button) = key_code_to_controller_button(key_code) {
              Some(Message::ControllerButtonChanged(button, true))
            } else {
              match key_code {
                KeyCode::R => Some(Message::EmulatorStateChangeRequested(EmulatorState::Run)),
                KeyCode::P => Some(Message::EmulatorStateChangeRequested(EmulatorState::Pause)),
                KeyCode::F => Some(Message::EmulatorStateChangeRequested(
                  EmulatorState::RunUntilNextFrame,
                )),
                KeyCode::I => Some(Message::EmulatorStateChangeRequested(
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
            .map(|button| Message::ControllerButtonChanged(button, false)),
          _ => None,
        },
        _ => None,
      }),
      iced::time::every(Duration::from_secs_f64(1.0 / 60.0)).map(|_| Message::Tick),
    ])
  }

  fn view(&self) -> iced::Element<'_, Self::Message> {
    let fps_text =
      text(format!("{:.02} FPS", 1.0 / self.last_tick_duration.as_secs_f32()).as_str())
        .font(PIXEL_NES_FONT)
        .size(20);
    let state_text = text(<&'static str>::from(self.emulator_state).to_string())
      .font(PIXEL_NES_FONT)
      .size(20);
    let registers_text = text(
      format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} S:{:02X}",
        self.machine.cpu.a,
        self.machine.cpu.x,
        self.machine.cpu.y,
        self.machine.cpu.get_status_register(),
        self.machine.cpu.s
      )
      .as_str(),
    )
    .font(PIXEL_NES_FONT)
    .size(20);

    let info_column = column![fps_text, state_text, registers_text].width(Length::FillPortion(1));

    let screen_view = image(self.crt_screen.image_handle())
      .width(Length::FillPortion(6))
      .height(Length::Fill);

    let layout = row![screen_view, info_column].spacing(20);

    layout.into()
  }
}
