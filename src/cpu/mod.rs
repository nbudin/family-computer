mod cpu;
mod cpu_bus;
mod disassembly;
mod instructions;
mod operand;

pub use cpu::*;
pub use cpu_bus::*;
pub use disassembly::*;
pub use instructions::*;
pub use operand::*;

#[cfg(test)]
mod tests {
  use similar_asserts::assert_eq;
  use std::{
    cell::RefCell,
    io::{BufReader, Write},
  };

  #[derive(Clone, Debug)]
  struct StringWriter {
    bytes: std::rc::Rc<RefCell<Vec<u8>>>,
  }

  impl StringWriter {
    pub fn new() -> Self {
      StringWriter {
        bytes: std::rc::Rc::new(RefCell::new(Vec::with_capacity(1 * 1024 * 1024))),
      }
    }

    pub fn into_string(self) -> String {
      String::from_utf8(self.bytes.take()).unwrap()
    }
  }

  impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
      self.bytes.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
      self.bytes.borrow_mut().flush()
    }
  }

  use crate::{gui::PIXEL_BUFFER_SIZE, ines_rom::INESRom, machine::Machine};

  #[test]
  fn nestest_smoke_test() {
    let nestest_data = include_bytes!("../../smoketest/nestest.nes");
    let expected_log = include_str!("../../smoketest/nestest-good.log");
    let rom = INESRom::from_reader(&mut BufReader::new(&nestest_data[..])).unwrap();

    let mut machine = Machine::from_rom(rom);
    // machine.cpu_cycle_count = 7;
    // machine.ppu.cycle = 21;
    machine.cpu.pc = 0xc000;
    machine.cpu.p = 0x24.into();

    let mut fake_pixbuf = [0; PIXEL_BUFFER_SIZE];
    let disasm_writer = StringWriter::new();
    machine.disassembly_writer = Some(Box::new(disasm_writer.clone()));

    // weird PPU behavior tests start here and I'm not sure those are valid
    while machine.cpu_cycle_count < 26520 {
      machine.tick(&mut fake_pixbuf);
    }

    let disasm: String = disasm_writer.into_string();

    for (line_index, (disasm_line, expected_line)) in disasm
      .split("\n")
      .zip(expected_log.split("\r\n"))
      .enumerate()
    {
      if !disasm_line.is_empty() {
        assert_eq!(
          disasm_line,
          expected_line,
          "Line {} did not match",
          line_index + 1
        );
      }
    }

    assert_eq!(
      disasm.split("\n").count(),
      8980,
      "Number of lines in disassembly log did not match"
    );
  }
}
