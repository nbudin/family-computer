mod apu;
mod apu_synth;
mod channel_state;
mod envelope;
mod length_counter;
mod linear_counter;
mod noise;
mod pulse;
mod registers;
mod sequencer;
mod sweep;
mod timing;
mod triangle;

pub use apu::*;
pub use apu_synth::*;
pub use channel_state::*;
pub use length_counter::*;
pub use noise::*;
pub use pulse::*;
pub use registers::*;
pub use sequencer::*;
pub use triangle::*;

#[cfg(test)]
mod tests {
  use std::io::BufReader;

  use crate::{
    nes::{INESRom, NES},
    ppu::Pixbuf,
  };

  fn run_blargg_apu_test(rom_data: &[u8]) -> Result<(), (u8, String)> {
    let rom = INESRom::from_reader(&mut BufReader::new(rom_data)).unwrap();
    let (sender, _receiver) = smol::channel::unbounded();
    let mut machine = NES::from_rom(rom, sender);
    let mut fake_pixbuf = Pixbuf::new();
    let mut result: u8;

    loop {
      machine.execute_frame(&mut fake_pixbuf);

      // blargg's ppu tests write their result to 0x00f8 in work ram
      result = machine.state.cartridge.cpu_bus().read_readonly(0x6000);

      // wait for a successful result or time out
      if result != 0x80 && result != 0x00 || machine.state.ppu.frame_count > 5 * 60 {
        break;
      }
    }

    let mut output_chars: Vec<u8> = vec![];
    let mut read_cursor = 0x6004;
    loop {
      let char = machine.state.cartridge.cpu_bus().read_readonly(read_cursor);
      if char == 0 {
        break;
      } else {
        output_chars.push(char);
        read_cursor += 1;
      }
    }

    Err((result, String::from_utf8_lossy(&output_chars).into_owned()))
  }

  #[test]
  fn test_apu_smoketest() {
    let rom_data = include_bytes!("../../smoketest/apu_test.nes");
    let result = run_blargg_apu_test(rom_data);

    if let Err((result, error_message)) = result {
      assert!(
        false,
        "Returned error code {:02X}: {}",
        result, error_message
      );
    }
  }
}
