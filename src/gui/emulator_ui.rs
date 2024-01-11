use std::{sync::Arc, time::Duration};

use fruitbasket::FruitApp;
use iced::{
  executor,
  theme::Palette,
  widget::{column, image, row, text, vertical_space},
  Application, Color, Command, Font, Length, Subscription, Theme,
};
use smol::channel::{Receiver, Sender};

use crate::{
  emulator::{
    EmulationInboundMessage, EmulationOutboundMessage, EmulatorBuilder, EmulatorState, MachineState,
  },
  nes::ControllerButton,
};

use super::{keys::handle_key_event, run_emulator, CRTScreen};

const PIXEL_NES_FONT: Font = Font::with_name("Pixel NES");

pub struct EmulatorUIFlags {
  emulator_builder: Box<dyn EmulatorBuilder>,
  #[cfg(target_os = "macos")]
  app: Option<FruitApp<'static>>,
}

impl EmulatorUIFlags {
  pub fn new(emulator_builder: Box<dyn EmulatorBuilder>) -> Self {
    Self {
      emulator_builder,
      #[cfg(target_os = "macos")]
      app: None,
    }
  }

  pub fn set_app(&mut self, app: FruitApp<'static>) {
    self.app = Some(app)
  }
}

#[derive(Debug, Clone)]
pub enum EmulatorUIMessage {
  ControllerButtonChanged(ControllerButton, bool),
  EmulatorStateChangeRequested(EmulatorState),
  FontLoaded(Result<(), iced::font::Error>),
  FrameReady,
  MachineStateChanged(MachineState),
}

pub struct EmulatorUI {
  crt_screen: CRTScreen,
  last_tick_duration: Duration,
  last_machine_state: MachineState,
  inbound_sender: Sender<EmulationInboundMessage>,
  outbound_receiver: Arc<Receiver<EmulationOutboundMessage>>,
  #[cfg(target_os = "macos")]
  app: fruitbasket::FruitApp<'static>,
}

impl Application for EmulatorUI {
  type Executor = executor::Default;
  type Message = EmulatorUIMessage;
  type Flags = EmulatorUIFlags;
  type Theme = Theme;

  fn new(flags: EmulatorUIFlags) -> (EmulatorUI, Command<Self::Message>) {
    let crt_screen = CRTScreen::new();
    let pixbuf = crt_screen.pixbuf.clone();
    let (inbound_sender, inbound_receiver) = smol::channel::unbounded();
    let (outbound_sender, outbound_receiver) = smol::channel::unbounded();

    (
      EmulatorUI {
        crt_screen,
        last_tick_duration: Duration::from_millis(1000),
        last_machine_state: MachineState::default(),
        inbound_sender,
        outbound_receiver: Arc::new(outbound_receiver),
        #[cfg(target_os = "macos")]
        app: flags.app.unwrap(),
      },
      Command::batch([
        Command::perform(
          run_emulator::run_emulator(
            flags.emulator_builder,
            pixbuf,
            inbound_receiver,
            outbound_sender,
          ),
          |_| EmulatorUIMessage::FrameReady,
        ),
        iced::font::load(include_bytes!("./Pixel_NES.otf").as_slice())
          .map(EmulatorUIMessage::FontLoaded),
      ]),
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
      EmulatorUIMessage::FontLoaded(_) => Command::none(),
      EmulatorUIMessage::ControllerButtonChanged(button, pressed) => {
        smol::block_on(async {
          self
            .inbound_sender
            .send(EmulationInboundMessage::ControllerButtonChanged(
              button, pressed,
            ))
            .await
        })
        .unwrap();
        Command::none()
      }
      EmulatorUIMessage::EmulatorStateChangeRequested(new_state) => {
        smol::block_on(async {
          self
            .inbound_sender
            .send(EmulationInboundMessage::EmulatorStateChangeRequested(
              new_state,
            ))
            .await
        })
        .unwrap();
        Command::none()
      }
      EmulatorUIMessage::FrameReady => {
        #[cfg(target_os = "macos")]
        self.app.run(fruitbasket::RunPeriod::Once).unwrap();

        Command::none()
      }
      EmulatorUIMessage::MachineStateChanged(machine_state) => {
        self.last_machine_state = machine_state;
        Command::none()
      }
    }
  }

  fn subscription(&self) -> Subscription<EmulatorUIMessage> {
    let outbound_receiver = self.outbound_receiver.clone();
    iced::Subscription::batch([
      iced::subscription::events_with(|event, _status| match event {
        iced::Event::Keyboard(event) => handle_key_event(event),
        _ => None,
      }),
      iced::subscription::unfold("emulator-outbound", (), move |()| {
        let outbound_receiver = outbound_receiver.clone();
        async move {
          let outbound_message = outbound_receiver.recv().await.unwrap();
          let ui_message = match outbound_message {
            EmulationOutboundMessage::FrameReady => EmulatorUIMessage::FrameReady,
            EmulationOutboundMessage::MachineStateChanged(state) => {
              EmulatorUIMessage::MachineStateChanged(state)
            }
          };

          (ui_message, ())
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
