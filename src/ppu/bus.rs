use crate::machine::Machine;

use super::{PPUAddressLatch, PPULoopyRegister, PPURegister, PPU};

impl PPU {
  pub fn read_bus(machine: &mut Machine, register: PPURegister) -> u8 {
    let mut result: u8 = 0;

    match register {
      PPURegister::PPUSTATUS => {
        result =
          (u8::from(machine.ppu.status) & 0b11100000) | (machine.ppu.data_buffer & 0b00011111);
        machine.ppu.status.set_vertical_blank(false);
        machine.ppu.address_latch = PPUAddressLatch::High;
        machine.ppu.status_register_read_this_tick = true;
      }
      PPURegister::PPUDATA => {
        result = machine.ppu.data_buffer;
        machine.ppu.data_buffer = machine
          .ppu
          .get_ppu_mem(machine, machine.ppu.vram_addr.into());

        if u16::from(machine.ppu.vram_addr) > 0x3f00 {
          // palette memory is read immediately
          result = machine.ppu.data_buffer;
        }

        machine.ppu.vram_addr = PPULoopyRegister::from(
          u16::from(machine.ppu.vram_addr)
            + if machine.ppu.control.increment_mode() {
              32
            } else {
              1
            },
        );
      }
      _ => {}
    }

    result
  }

  pub fn read_bus_readonly(machine: &Machine, register: PPURegister) -> u8 {
    match register {
      PPURegister::PPUSTATUS => {
        (u8::from(machine.ppu.status) & 0b11100000) | (machine.ppu.data_buffer & 0b00011111)
      }
      PPURegister::PPUDATA => {
        if u16::from(machine.ppu.vram_addr) > 0x3f00 {
          // palette memory is read immediately
          machine
            .ppu
            .get_ppu_mem(machine, machine.ppu.vram_addr.into())
        } else {
          machine.ppu.data_buffer
        }
      }
      _ => 0,
    }
  }

  pub fn write_bus(machine: &mut Machine, register: PPURegister, value: u8) {
    match register {
      PPURegister::PPUCTRL => {
        machine.ppu.control = value.into();
        machine
          .ppu
          .tram_addr
          .set_nametable_x(machine.ppu.control.nametable_x());
        machine
          .ppu
          .tram_addr
          .set_nametable_y(machine.ppu.control.nametable_y());
      }
      PPURegister::PPUMASK => {
        machine.ppu.mask = value.into();
      }
      PPURegister::PPUSCROLL => match machine.ppu.address_latch {
        PPUAddressLatch::High => {
          machine.ppu.fine_x = value & 0x07;
          machine.ppu.tram_addr.set_coarse_x(value >> 3);
          machine.ppu.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          machine.ppu.tram_addr.set_fine_y(value & 0x07);
          machine.ppu.tram_addr.set_coarse_y(value >> 3);
          machine.ppu.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUADDR => match machine.ppu.address_latch {
        PPUAddressLatch::High => {
          machine.ppu.tram_addr = PPULoopyRegister::from(
            (u16::from(machine.ppu.tram_addr) & 0x00ff) | (u16::from(value) << 8),
          );
          machine.ppu.address_latch = PPUAddressLatch::Low;
        }
        PPUAddressLatch::Low => {
          machine.ppu.tram_addr =
            PPULoopyRegister::from((u16::from(machine.ppu.tram_addr) & 0xff00) | u16::from(value));
          machine.ppu.vram_addr = machine.ppu.tram_addr;
          machine.ppu.address_latch = PPUAddressLatch::High;
        }
      },
      PPURegister::PPUDATA => {
        PPU::set_ppu_mem(machine, u16::from(machine.ppu.vram_addr), value);
        machine.ppu.vram_addr = PPULoopyRegister::from(
          u16::from(machine.ppu.vram_addr)
            + if machine.ppu.control.increment_mode() {
              32
            } else {
              1
            },
        );
      }
      _ => {}
    }
  }
}
