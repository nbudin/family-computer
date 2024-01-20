use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use crate::{bus::Bus, cpu::CPUBusTrait};

pub enum InterceptorResult<T> {
  Intercepted(T),
  NotIntercepted,
}

pub trait BusInterceptor<AddrType: Clone>: Send + Sync {
  type BusType: Bus<AddrType>;

  fn get_inner(&self) -> &Self::BusType;
  fn get_inner_mut(&mut self) -> &mut Self::BusType;

  fn intercept_read_readonly(&self, addr: AddrType) -> InterceptorResult<Option<u8>>;
  fn intercept_write(&mut self, addr: AddrType, value: u8) -> InterceptorResult<()>;

  fn intercept_read_side_effects(&mut self, _addr: AddrType) -> InterceptorResult<()> {
    InterceptorResult::NotIntercepted
  }
}

impl<AddrType: Clone, I: BusInterceptor<AddrType> + ?Sized> Bus<AddrType> for I {
  fn try_read_readonly(&self, addr: AddrType) -> Option<u8> {
    match self.intercept_read_readonly(addr.clone()) {
      InterceptorResult::Intercepted(value) => value,
      InterceptorResult::NotIntercepted => self.get_inner().try_read_readonly(addr),
    }
  }

  fn read_side_effects(&mut self, addr: AddrType) {
    match self.intercept_read_side_effects(addr.clone()) {
      InterceptorResult::Intercepted(_) => {}
      InterceptorResult::NotIntercepted => self.get_inner_mut().read_side_effects(addr),
    }
  }

  fn write(&mut self, addr: AddrType, value: u8) {
    match self.intercept_write(addr.clone(), value) {
      InterceptorResult::Intercepted(_) => {}
      InterceptorResult::NotIntercepted => self.get_inner_mut().write(addr, value),
    }
  }
}

#[derive(Debug, Clone)]
pub struct PassthroughBusInterceptor<AddrType: Clone, BusType: Bus<AddrType>> {
  bus: BusType,
  _phantom: PhantomData<AddrType>,
}

impl<AddrType: Clone, BusType: Bus<AddrType>> PassthroughBusInterceptor<AddrType, BusType> {
  pub fn new(bus: BusType) -> Self {
    Self {
      bus,
      _phantom: Default::default(),
    }
  }
}

impl<AddrType: Clone + Send + Sync, BusType: Bus<AddrType> + Clone> BusInterceptor<AddrType>
  for PassthroughBusInterceptor<AddrType, BusType>
{
  type BusType = BusType;

  fn get_inner(&self) -> &BusType {
    &self.bus
  }

  fn get_inner_mut(&mut self) -> &mut BusType {
    &mut self.bus
  }

  fn intercept_read_readonly(&self, _addr: AddrType) -> InterceptorResult<Option<u8>> {
    InterceptorResult::NotIntercepted
  }

  fn intercept_write(&mut self, _addr: AddrType, _value: u8) -> InterceptorResult<()> {
    InterceptorResult::NotIntercepted
  }
}

impl<BusType: CPUBusTrait, I: BusInterceptor<u16, BusType = BusType>> CPUBusTrait for I {
  fn maybe_tick_dma(&mut self, ppu_cycle_count: u64) -> bool {
    self.get_inner_mut().maybe_tick_dma(ppu_cycle_count)
  }

  fn tick_apu(
    &mut self,
    sender: &<crate::apu::APUSynth as crate::audio::stream_setup::StreamSpawner>::OutputType,
    cpu_cycle_count: u64,
  ) -> bool {
    self.get_inner_mut().tick_apu(sender, cpu_cycle_count)
  }

  fn set_controller_button_state(
    &mut self,
    controller_index: usize,
    button: crate::nes::ControllerButton,
    pressed: bool,
  ) {
    self
      .get_inner_mut()
      .set_controller_button_state(controller_index, button, pressed)
  }
}

// impl<
//     AddrType: Clone + Send + Sync,
//     I: BusInterceptor<AddrType> + ?Sized,
//     T: Deref<Target = I> + DerefMut<Target = I> + Send + Sync + ?Sized,
//   > BusInterceptor<AddrType> for T
// {
//   type BusType = I::BusType;

//   fn get_inner(&self) -> &Self::BusType {
//     self.deref().get_inner()
//   }

//   fn get_inner_mut(&mut self) -> &mut Self::BusType {
//     self.deref_mut().get_inner_mut()
//   }

//   fn intercept_read_readonly(&self, addr: AddrType) -> InterceptorResult<Option<u8>> {
//     self.deref().intercept_read_readonly(addr)
//   }

//   fn intercept_read_side_effects(&mut self, addr: AddrType) -> InterceptorResult<()> {
//     self.deref_mut().intercept_read_side_effects(addr)
//   }

//   fn intercept_write(&mut self, addr: AddrType, value: u8) -> InterceptorResult<()> {
//     self.deref_mut().intercept_write(addr, value)
//   }
// }

#[cfg(test)]
mod tests {
  pub use super::*;

  #[derive(Clone)]
  pub struct AlwaysReturn5 {
    pub address_42_read: bool,
    pub last_written_value: u8,
  }

  impl AlwaysReturn5 {
    pub fn new() -> Self {
      AlwaysReturn5 {
        address_42_read: false,
        last_written_value: 0,
      }
    }
  }

  impl Bus<usize> for AlwaysReturn5 {
    fn try_read_readonly(&self, _addr: usize) -> Option<u8> {
      Some(5)
    }

    fn read_side_effects(&mut self, addr: usize) {
      if addr == 42 {
        self.address_42_read = true;
      }
    }

    fn write(&mut self, _addr: usize, value: u8) {
      self.last_written_value = value;
    }
  }

  #[derive(Clone)]
  pub struct InterceptIfGreaterThan42 {
    pub bus: AlwaysReturn5,
    pub address_142_read: bool,
    pub last_written_value: u8,
  }

  impl InterceptIfGreaterThan42 {
    pub fn new() -> Self {
      InterceptIfGreaterThan42 {
        bus: AlwaysReturn5::new(),
        address_142_read: false,
        last_written_value: 0,
      }
    }
  }

  impl BusInterceptor<usize> for InterceptIfGreaterThan42 {
    type BusType = AlwaysReturn5;

    fn get_inner(&self) -> &AlwaysReturn5 {
      &self.bus
    }

    fn get_inner_mut(&mut self) -> &mut AlwaysReturn5 {
      &mut self.bus
    }

    fn intercept_read_readonly(&self, addr: usize) -> InterceptorResult<Option<u8>> {
      if addr > 42 {
        InterceptorResult::Intercepted(Some(99))
      } else {
        InterceptorResult::NotIntercepted
      }
    }

    fn intercept_read_side_effects(&mut self, addr: usize) -> InterceptorResult<()> {
      if addr > 42 {
        if addr == 142 {
          self.address_142_read = true;
        }
        InterceptorResult::Intercepted(())
      } else {
        InterceptorResult::NotIntercepted
      }
    }

    fn intercept_write(&mut self, addr: usize, value: u8) -> InterceptorResult<()> {
      if addr > 42 {
        self.last_written_value = value;
        InterceptorResult::Intercepted(())
      } else {
        InterceptorResult::NotIntercepted
      }
    }
  }

  #[test]
  fn test_read_readonly() {
    let interceptor = InterceptIfGreaterThan42::new();

    assert_eq!(interceptor.read_readonly(1), 5);
    assert_eq!(interceptor.read_readonly(42), 5);
    assert!(!interceptor.bus.address_42_read);
    assert_eq!(interceptor.read_readonly(43), 99);
    assert_eq!(interceptor.read_readonly(142), 99);
    assert!(!interceptor.address_142_read);
  }

  #[test]
  fn test_read() {
    let mut interceptor = InterceptIfGreaterThan42::new();

    assert_eq!(interceptor.read(1), 5);
    assert_eq!(interceptor.read(42), 5);
    assert!(interceptor.bus.address_42_read);
    assert_eq!(interceptor.read(43), 99);
    assert_eq!(interceptor.read(142), 99);
    assert!(interceptor.address_142_read);
  }

  #[test]
  fn test_write() {
    let mut interceptor = InterceptIfGreaterThan42::new();

    interceptor.write(42, 20);
    interceptor.write(142, 80);
    assert_eq!(interceptor.bus.last_written_value, 20);
    assert_eq!(interceptor.last_written_value, 80);
    assert!(!interceptor.bus.address_42_read);
    assert!(!interceptor.address_142_read);
  }
}
