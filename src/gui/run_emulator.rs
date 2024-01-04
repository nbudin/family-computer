use std::sync::{Arc, RwLock};

use smol::channel::{Receiver, Sender};

use crate::{
  emulator::{EmulationInboundMessage, EmulationOutboundMessage, EmulatorBuilder},
  ppu::Pixbuf,
};

pub async fn run_emulator(
  builder: Box<dyn EmulatorBuilder>,
  pixbuf: Arc<RwLock<Pixbuf>>,
  inbound_receiver: Receiver<EmulationInboundMessage>,
  outbound_sender: Sender<EmulationOutboundMessage>,
) {
  let mut emulator = builder.build(pixbuf);
  emulator.run(inbound_receiver, outbound_sender).await
}
