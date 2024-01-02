use std::{
  convert::Infallible,
  sync::{Arc, RwLock},
  thread,
  time::{Duration, Instant},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use strum::IntoStaticStr;

use crate::{
  bus::Bus,
  controller::ControllerButton,
  cpu::CPU,
  machine::Machine,
  ppu::{PPULoopyRegister, Pixbuf},
};

const TARGET_FRAME_DURATION: f64 = 1.0 / 60.0;

#[derive(Debug, Clone, Copy, IntoStaticStr, Default)]
pub enum EmulatorState {
  Run,
  #[default]
  Pause,
  RunUntilNextFrame,
  RunUntilNextInstruction,
}

#[derive(Default, Debug, Clone)]
pub struct MachineState {
  pub emulator_state: EmulatorState,
  pub cpu: CPU,
  pub vram_addr: PPULoopyRegister,
  pub tram_addr: PPULoopyRegister,
  pub scanline: i32,
  pub cycle: i32,
  pub mem2002: u8,
  pub mem2004: u8,
  pub mem2007: u8,
}

pub enum EmulationInboundMessage {
  ControllerButtonChanged(ControllerButton, bool),
  EmulatorStateChangeRequested(EmulatorState),
}

#[derive(Debug)]
pub enum EmulationOutboundMessage {
  FrameReady,
  MachineStateChanged(MachineState),
}

pub struct Emulator {
  machine: Machine,
  state: EmulatorState,
  last_tick: Instant,
  pixbuf: Arc<RwLock<Pixbuf>>,
  inbound_receiver: Receiver<EmulationInboundMessage>,
}

impl Emulator {
  pub fn new(
    machine: Machine,
    pixbuf: Arc<RwLock<Pixbuf>>,
  ) -> (Self, Sender<EmulationInboundMessage>) {
    let (inbound_sender, inbound_receiver) = unbounded();
    let emulator = Self {
      machine,
      state: EmulatorState::Pause,
      last_tick: Instant::now(),
      pixbuf,
      inbound_receiver,
    };
    (emulator, inbound_sender)
  }

  fn get_machine_state(&self) -> MachineState {
    let cpu_bus = self.machine.cpu_bus();

    MachineState {
      emulator_state: self.state,
      cpu: self.machine.cpu.clone(),
      vram_addr: self.machine.ppu.vram_addr,
      tram_addr: self.machine.ppu.tram_addr,
      scanline: self.machine.ppu.scanline,
      cycle: self.machine.ppu.cycle,
      mem2002: cpu_bus.read_readonly(0x2002),
      mem2004: cpu_bus.read_readonly(0x2004),
      mem2007: cpu_bus.read_readonly(0x2007),
    }
  }

  pub fn run(
    &mut self,
    mut sender: iced::futures::channel::mpsc::Sender<EmulationOutboundMessage>,
  ) -> Infallible {
    loop {
      while let Ok(message) = self.inbound_receiver.try_recv() {
        match message {
          EmulationInboundMessage::ControllerButtonChanged(button, pressed) => {
            self.machine.controllers[0].set_button_state(button, pressed)
          }
          EmulationInboundMessage::EmulatorStateChangeRequested(new_state) => {
            self.state = new_state
          }
        }
      }

      let now = Instant::now();
      let wait_duration =
        Duration::from_secs_f64(TARGET_FRAME_DURATION).saturating_sub(now - self.last_tick);
      thread::sleep(wait_duration);
      self.last_tick = Instant::now();

      match self.state {
        EmulatorState::Pause => {}
        EmulatorState::Run => {
          self
            .machine
            .execute_frame(&mut self.pixbuf.write().unwrap());
          sender
            .try_send(EmulationOutboundMessage::MachineStateChanged(
              self.get_machine_state(),
            ))
            .unwrap();
          sender
            .try_send(EmulationOutboundMessage::FrameReady)
            .unwrap();
        }
        EmulatorState::RunUntilNextFrame => {
          self
            .machine
            .execute_frame(&mut self.pixbuf.write().unwrap());
          sender
            .try_send(EmulationOutboundMessage::MachineStateChanged(
              self.get_machine_state(),
            ))
            .unwrap();
          sender
            .try_send(EmulationOutboundMessage::FrameReady)
            .unwrap();
          self.state = EmulatorState::Pause;
        }
        EmulatorState::RunUntilNextInstruction => {
          let start_cycles = self.machine.cpu_cycle_count;
          loop {
            self.machine.tick(&mut self.pixbuf.write().unwrap());

            if self.machine.cpu_cycle_count > start_cycles {
              break;
            }
          }
          sender
            .try_send(EmulationOutboundMessage::MachineStateChanged(
              self.get_machine_state(),
            ))
            .unwrap();
          sender
            .try_send(EmulationOutboundMessage::FrameReady)
            .unwrap();

          self.state = EmulatorState::Pause;
        }
      }
    }
  }
}
