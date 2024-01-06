use std::sync::{Arc, RwLock};

use smol::channel::{Receiver, Sender};

use crate::{
  apu::APUSynth,
  audio::stream_setup::stream_setup_for,
  emulator::{EmulationInboundMessage, EmulationOutboundMessage, EmulatorBuilder},
  ppu::Pixbuf,
};

pub async fn run_emulator(
  builder: Box<dyn EmulatorBuilder>,
  pixbuf: Arc<RwLock<Pixbuf>>,
  inbound_receiver: Receiver<EmulationInboundMessage>,
  outbound_sender: Sender<EmulationOutboundMessage>,
) {
  let apu_synth = APUSynth::new();
  let apu_sender = stream_setup_for(apu_synth).unwrap();

  let mut emulator = builder.build(pixbuf, apu_sender);
  emulator.run(inbound_receiver, outbound_sender).await
}
