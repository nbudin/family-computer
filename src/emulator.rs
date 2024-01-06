use std::{
  env,
  io::BufWriter,
  sync::{Arc, RwLock},
  time::{Duration, Instant},
};

use smol::channel::{Receiver, Sender};
use smol::stream::StreamExt;
use strum::IntoStaticStr;

use crate::{
  apu::APUSynthChannel,
  audio::synth::SynthCommand,
  bus::Bus,
  controller::ControllerButton,
  cpu::CPU,
  ines_rom::INESRom,
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

#[derive(Debug)]
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
}

impl Emulator {
  pub fn new(machine: Machine, pixbuf: Arc<RwLock<Pixbuf>>) -> Self {
    let emulator = Self {
      machine,
      state: EmulatorState::Run,
      last_tick: Instant::now(),
      pixbuf,
    };
    emulator
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

  pub async fn run(
    &mut self,
    inbound_receiver: Receiver<EmulationInboundMessage>,
    outbound_sender: Sender<EmulationOutboundMessage>,
  ) {
    let mut timer = smol::Timer::interval(Duration::from_secs_f64(TARGET_FRAME_DURATION));

    loop {
      timer.next().await;
      self.run_once(&inbound_receiver, &outbound_sender).await;
    }
  }

  pub async fn run_once(
    &mut self,
    receiver: &Receiver<EmulationInboundMessage>,
    sender: &Sender<EmulationOutboundMessage>,
  ) {
    while let Ok(message) = receiver.try_recv() {
      match message {
        EmulationInboundMessage::ControllerButtonChanged(button, pressed) => {
          self.machine.controllers[0].set_button_state(button, pressed)
        }
        EmulationInboundMessage::EmulatorStateChangeRequested(new_state) => self.state = new_state,
      }
    }

    match self.state {
      EmulatorState::Pause => {}
      EmulatorState::Run => {
        self
          .machine
          .execute_frame(&mut self.pixbuf.write().unwrap());
        sender
          .send(EmulationOutboundMessage::MachineStateChanged(
            self.get_machine_state(),
          ))
          .await
          .unwrap();
        sender
          .send(EmulationOutboundMessage::FrameReady)
          .await
          .unwrap();
      }
      EmulatorState::RunUntilNextFrame => {
        self
          .machine
          .execute_frame(&mut self.pixbuf.write().unwrap());
        sender
          .send(EmulationOutboundMessage::MachineStateChanged(
            self.get_machine_state(),
          ))
          .await
          .unwrap();
        sender
          .send(EmulationOutboundMessage::FrameReady)
          .await
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
          .send(EmulationOutboundMessage::MachineStateChanged(
            self.get_machine_state(),
          ))
          .await
          .unwrap();
        sender
          .send(EmulationOutboundMessage::FrameReady)
          .await
          .unwrap();

        self.state = EmulatorState::Pause;
      }
    }
  }
}

pub trait EmulatorBuilder: Send + Sync {
  fn build(
    &self,
    pixbuf: Arc<RwLock<Pixbuf>>,
    apu_sender: Sender<SynthCommand<APUSynthChannel>>,
  ) -> Emulator;
}

pub struct NESEmulatorBuilder {
  rom: INESRom,
}

impl NESEmulatorBuilder {
  pub fn new(rom: INESRom) -> Self {
    Self { rom }
  }
}

impl EmulatorBuilder for NESEmulatorBuilder {
  fn build(
    &self,
    pixbuf: Arc<RwLock<Pixbuf>>,
    apu_sender: Sender<SynthCommand<APUSynthChannel>>,
  ) -> Emulator {
    let mut machine = Machine::from_rom(self.rom.clone(), apu_sender);
    let stdout = std::io::stdout();

    if !env::var("DISASSEMBLE").unwrap_or_default().is_empty() {
      let disassembly_writer = BufWriter::new(stdout);
      machine.disassembly_writer = Some(Arc::new(RwLock::new(disassembly_writer)));
    }

    Emulator::new(machine, pixbuf)
  }
}
