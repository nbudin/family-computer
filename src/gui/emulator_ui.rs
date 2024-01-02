use std::{
  convert::Infallible,
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

use iced::{
  executor,
  futures::channel::mpsc::Sender,
  keyboard::{self, KeyCode},
  theme::Palette,
  widget::{column, image, row, text, vertical_space},
  Application, Color, Command, Font, Length, Subscription, Theme,
};

use crate::{
  controller::ControllerButton,
  emulator::{
    EmulationInboundMessage, EmulationOutboundMessage, Emulator, EmulatorState, MachineState,
  },
  machine::Machine,
};

use super::CRTScreen;

const PIXEL_NES_FONT: Font = Font::with_name("Pixel NES");

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
  emulator: Arc<Mutex<Emulator>>,
  emulator_sender: crossbeam_channel::Sender<EmulationInboundMessage>,
  crt_screen: CRTScreen,
  last_tick_duration: Duration,
  last_machine_state: MachineState,
}

#[derive(Debug, Clone)]
pub enum Message {
  ControllerButtonChanged(ControllerButton, bool),
  EmulatorStateChangeRequested(EmulatorState),
  FontLoaded(Result<(), iced::font::Error>),
  FrameReady,
  MachineStateChanged(MachineState),
}

impl Application for EmulatorUI {
  type Executor = executor::Default;
  type Message = Message;
  type Flags = EmulatorUIFlags;
  type Theme = Theme;

  fn new(flags: EmulatorUIFlags) -> (EmulatorUI, Command<Self::Message>) {
    let crt_screen = CRTScreen::new();
    let (emulator, emulator_sender) = Emulator::new(flags.machine, crt_screen.pixbuf.clone());

    (
      EmulatorUI {
        emulator: Arc::new(Mutex::new(emulator)),
        emulator_sender,
        crt_screen,
        last_tick_duration: Duration::from_millis(1000),
        last_machine_state: MachineState::default(),
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
        self
          .emulator_sender
          .send(EmulationInboundMessage::ControllerButtonChanged(
            button, pressed,
          ))
          .unwrap();
        Command::none()
      }
      Message::EmulatorStateChangeRequested(new_state) => {
        self
          .emulator_sender
          .send(EmulationInboundMessage::EmulatorStateChangeRequested(
            new_state,
          ))
          .unwrap();
        Command::none()
      }
      Message::FrameReady => {
        // TODO do we need to actually do anything here?
        Command::none()
      }
      Message::MachineStateChanged(machine_state) => {
        self.last_machine_state = machine_state;
        Command::none()
      }
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    fn handle_key_event(event: iced::keyboard::Event) -> Option<Message> {
      match event {
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
      }
    }

    async fn run_emulator_async(
      emulator: Arc<Mutex<Emulator>>,
      sender: Sender<EmulationOutboundMessage>,
    ) -> Infallible {
      thread::spawn(move || emulator.lock().unwrap().run(sender))
        .join()
        .unwrap()
    }

    let emulator = self.emulator.clone();

    iced::Subscription::batch([
      iced::subscription::events_with(|event, _status| match event {
        iced::Event::Keyboard(event) => handle_key_event(event),
        _ => None,
      }),
      iced::subscription::channel("emulator-outbound", 64, |sender| {
        run_emulator_async(emulator, sender)
      })
      .map(|emulation_message| {
        println!("{:?}", emulation_message);

        match emulation_message {
          EmulationOutboundMessage::FrameReady => Message::FrameReady,
          EmulationOutboundMessage::MachineStateChanged(machine_state) => {
            Message::MachineStateChanged(machine_state)
          }
        }
      }),
    ])
  }

  fn view(&self) -> iced::Element<'_, Self::Message> {
    let fps_text =
      text(format!("{:.02} FPS", 1.0 / self.last_tick_duration.as_secs_f32()).as_str())
        .font(PIXEL_NES_FONT)
        .size(20);
    let state_text = text(<&'static str>::from(self.last_machine_state.emulator_state).to_string())
      .font(PIXEL_NES_FONT)
      .size(20);
    let machine = &self.last_machine_state;
    let registers_text = text(
      format!(
        "A-{:02X} X-{:02X} Y-{:02X} S-{:02X}\nPC-{:04X}",
        machine.cpu.a, machine.cpu.x, machine.cpu.y, machine.cpu.s, machine.cpu.pc
      )
      .as_str(),
    )
    .font(PIXEL_NES_FONT)
    .size(20);
    let cpu_status_text = text(
      format!(
        "N-{} V-{} D-{} I-{} Z-{} C-{}",
        u8::from(machine.cpu.p.negative_flag()),
        u8::from(machine.cpu.p.overflow_flag()),
        u8::from(machine.cpu.p.decimal_flag()),
        u8::from(machine.cpu.p.interrupt_disable()),
        u8::from(machine.cpu.p.zero_flag()),
        u8::from(machine.cpu.p.carry_flag()),
      )
      .as_str(),
    )
    .font(PIXEL_NES_FONT)
    .size(20);

    let ppu_status_text = text(
      format!(
        "Scanl {}\nCycle {}\n$2002-{:02X}\n$2004-{:02X}\n$2007-{:02X}\nv:{:04X} t:{:04X}",
        machine.scanline,
        machine.cycle,
        machine.mem2002,
        machine.mem2004,
        machine.mem2007,
        u16::from(machine.vram_addr),
        u16::from(machine.tram_addr)
      )
      .as_str(),
    )
    .font(PIXEL_NES_FONT)
    .size(20);

    let info_column = column![
      fps_text,
      state_text,
      registers_text,
      cpu_status_text,
      ppu_status_text,
      vertical_space(10),
    ]
    .width(Length::FillPortion(1));

    let screen_view = image(self.crt_screen.image_handle())
      .width(Length::FillPortion(4))
      .height(Length::Fill);

    let layout = row![screen_view, info_column].spacing(20);

    layout.into()
  }
}
