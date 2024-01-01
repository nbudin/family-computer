use crate::bus::Bus;

pub enum InterceptorResult<T> {
  Intercepted(T),
  NotIntercepted,
}

pub trait BusInterceptor<'a, AddrType: Clone> {
  fn bus(&self) -> &dyn Bus<AddrType>;
  fn bus_mut(&mut self) -> &mut dyn Bus<AddrType>;

  fn intercept_read_readonly(&self, addr: AddrType) -> InterceptorResult<Option<u8>>;
  fn intercept_write(&mut self, addr: AddrType, value: u8) -> InterceptorResult<()>;

  fn intercept_read_side_effects(&mut self, _addr: AddrType) -> InterceptorResult<()> {
    InterceptorResult::NotIntercepted
  }
}

impl<'a, AddrType: Clone, I: BusInterceptor<'a, AddrType> + ?Sized> Bus<AddrType> for I {
  fn try_read_readonly(&self, addr: AddrType) -> Option<u8> {
    match self.intercept_read_readonly(addr.clone()) {
      InterceptorResult::Intercepted(value) => value,
      InterceptorResult::NotIntercepted => self.bus().try_read_readonly(addr),
    }
  }

  fn read_side_effects(&mut self, addr: AddrType) {
    match self.intercept_read_side_effects(addr.clone()) {
      InterceptorResult::Intercepted(_) => {}
      InterceptorResult::NotIntercepted => self.bus_mut().read_side_effects(addr),
    }
  }

  fn write(&mut self, addr: AddrType, value: u8) {
    match self.intercept_write(addr.clone(), value) {
      InterceptorResult::Intercepted(_) => {}
      InterceptorResult::NotIntercepted => self.bus_mut().write(addr, value),
    }
  }
}

#[cfg(test)]
mod tests {
  pub use super::*;

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

  impl BusInterceptor<'_, usize> for InterceptIfGreaterThan42 {
    fn bus(&self) -> &dyn Bus<usize> {
      &self.bus
    }

    fn bus_mut(&mut self) -> &mut dyn Bus<usize> {
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
