mod bus;
mod drawing;
mod ppu;
mod ppu_memory;
mod registers;
mod scrolling;

pub use ppu::*;
pub use registers::*;

#[cfg(test)]
mod tests {
  use std::io::BufReader;

  use crate::{gfx::crt_screen::PIXEL_BUFFER_SIZE, ines_rom::INESRom, machine::Machine};

  fn run_blargg_ppu_test(rom_data: &[u8]) -> u8 {
    let rom = INESRom::from_reader(&mut BufReader::new(&rom_data[..])).unwrap();
    let mut machine = Machine::from_rom(rom);
    let mut fake_pixbuf = [0; PIXEL_BUFFER_SIZE];
    let mut result: u8;

    loop {
      machine.execute_frame(&mut fake_pixbuf);

      // blargg's ppu tests write their result to 0x00f8 in work ram
      result = machine.get_cpu_mem_readonly(0x00f8);
      if result != 0 || machine.ppu.frame_count > 10 * 60 {
        break;
      }
    }

    result
  }

  #[test]
  fn test_1_frame_basics() {
    let rom_data = include_bytes!("../../smoketest/1.frame_basics.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }

  #[test]
  fn test_2_vbl_timing() {
    let rom_data = include_bytes!("../../smoketest/2.vbl_timing.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }

  #[test]
  fn test_3_even_odd_frames() {
    let rom_data = include_bytes!("../../smoketest/3.even_odd_frames.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }

  #[test]
  fn test_4_vbl_clear_timing() {
    let rom_data = include_bytes!("../../smoketest/4.vbl_clear_timing.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }

  #[test]
  fn test_5_nmi_suppression() {
    let rom_data = include_bytes!("../../smoketest/5.nmi_suppression.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }

  #[test]
  fn test_6_nmi_disable() {
    let rom_data = include_bytes!("../../smoketest/6.nmi_disable.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }

  #[test]
  fn test_7_nmi_timing() {
    let rom_data = include_bytes!("../../smoketest/7.nmi_timing.nes");
    let result = run_blargg_ppu_test(rom_data);

    assert!(result == 1, "Returned error code {}", result);
  }
}
