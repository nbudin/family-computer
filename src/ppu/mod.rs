mod drawing;
mod palette;
mod pixbuf;
mod ppu;
mod ppu_cpu_bus;
mod ppu_memory;
mod registers;
mod scrolling;
mod sprites;

pub use pixbuf::*;
pub use ppu::*;
pub use ppu_cpu_bus::*;
pub use ppu_memory::*;
pub use registers::*;
pub use sprites::*;

#[cfg(test)]
mod tests {
  use std::io::BufReader;

  use crate::{
    bus::Bus,
    nes::{INESRom, NES},
  };

  use super::Pixbuf;

  fn run_blargg_ppu_test(rom_data: &[u8]) -> u8 {
    let rom = INESRom::from_reader(&mut BufReader::new(&rom_data[..])).unwrap();
    let (sender, _receiver) = smol::channel::unbounded();
    let mut machine = NES::from_rom(rom, sender);
    let mut fake_pixbuf = Pixbuf::new();
    let mut result: u8;

    loop {
      machine.execute_frame(&mut fake_pixbuf);

      // blargg's ppu tests write their result to 0x00f8 in work ram
      result = machine.cpu_bus().read_readonly(0x00f8);

      // wait for a successful result or time out
      if result == 1 || machine.ppu.frame_count > 5 * 60 {
        break;
      }
    }

    result
  }

  #[test]
  fn test_1_frame_basics() {
    let rom_data = include_bytes!("../../smoketest/1.frame_basics.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "VBL flag isn't being set",
      3 => "VBL flag should be cleared after being read",
      4 => "PPU frame with BG enabled is too short",
      5 => "PPU frame with BG enabled is too long",
      6 => "PPU frame with BG disabled is too short",
      7 => "PPU frame with BG disabled is too long",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }

  #[test]
  fn test_2_vbl_timing() {
    let rom_data = include_bytes!("../../smoketest/2.vbl_timing.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "Flag should read as clear 3 PPU clocks before VBL",
      3 => "Flag should read as set 0 PPU clocks after VBL",
      4 => "Flag should read as clear 2 PPU clocks before VBL",
      5 => "Flag should read as set 1 PPU clock after VBL",
      6 => "Flag should read as clear 1 PPU clock before VBL",
      7 => "Flag should read as set 2 PPU clocks after VBL",
      8 => "Reading 1 PPU clock before VBL should suppress setting",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }

  #[test]
  fn test_3_even_odd_frames() {
    let rom_data = include_bytes!("../../smoketest/3.even_odd_frames.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "Pattern ----- should not skip any clocks",
      3 => "Pattern BB--- should skip 1 clock",
      4 => "Pattern B--B- (one even, one odd) should skip 1 clock",
      5 => "Pattern -B--B (one odd, one even) should skip 1 clock",
      6 => "Pattern BB-BB (two pairs) should skip 2 clocks",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }

  #[test]
  fn test_4_vbl_clear_timing() {
    let rom_data = include_bytes!("../../smoketest/4.vbl_clear_timing.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "Cleared 3 or more PPU clocks too early",
      3 => "Cleared 2 PPU clocks too early",
      4 => "Cleared 1 PPU clock too early ",
      5 => "Cleared 3 or more PPU clocks too late",
      6 => "Cleared 2 PPU clocks too late",
      7 => "Cleared 1 PPU clock too late",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }

  #[test]
  fn test_5_nmi_suppression() {
    let rom_data = include_bytes!("../../smoketest/5.nmi_suppression.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "Reading flag 3 PPU clocks before set shouldn't suppress NMI",
      3 => "Reading flag when it's set should suppress NMI",
      4 => "Reading flag 3 PPU clocks after set shouldn't suppress NMI",
      5 => "Reading flag 2 PPU clocks before set shouldn't suppress NMI",
      6 => "Reading flag 1 PPU clock after set should suppress NMI",
      7 => "Reading flag 4 PPU clocks after set shouldn't suppress NMI",
      8 => "Reading flag 4 PPU clocks before set shouldn't suppress NMI",
      9 => "Reading flag 1 PPU clock before set should suppress NMI",
      10 => "Reading flag 2 PPU clocks after set shouldn't suppress NMI",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }

  #[test]
  fn test_6_nmi_disable() {
    let rom_data = include_bytes!("../../smoketest/6.nmi_disable.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "NMI shouldn't occur when disabled 0 PPU clocks after VBL",
      3 => "NMI should occur when disabled 3 PPU clocks after VBL",
      4 => "NMI shouldn't occur when disabled 1 PPU clock after VBL",
      5 => "NMI should occur when disabled 4 PPU clocks after VBL",
      6 => "NMI shouldn't occur when disabled 1 PPU clock before VBL",
      7 => "NMI should occur when disabled 2 PPU clocks after VBL",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }

  #[test]
  fn test_7_nmi_timing() {
    let rom_data = include_bytes!("../../smoketest/7.nmi_timing.nes");
    let result = run_blargg_ppu_test(rom_data);

    let error_message = match result {
      2 => "NMI occurred 3 or more PPU clocks too early",
      3 => "NMI occurred 2 PPU clocks too early",
      4 => "NMI occurred 1 PPU clock too early",
      5 => "NMI occurred 3 or more PPU clocks too late",
      6 => "NMI occurred 2 PPU clocks too late",
      7 => "NMI occurred 1 PPU clock too late",
      8 => "NMI should occur if enabled when VBL already set",
      9 => "NMI enabled when VBL already set should delay 1 instruction",
      10 => "NMI should be possible multiple times in VBL",
      _ => "",
    };

    assert!(
      result == 1,
      "Returned error code {}: {}",
      result,
      error_message
    );
  }
}
