use super::APUTimerRegister;

#[derive(Debug, Clone)]
pub struct APUSweep {
  pub enabled: bool,
  pub shift_count: u8,
  pub negate: bool,
  pub output: APUTimerRegister,
  pub ones_complement_negate: bool,
  pub divider_period: u8,
  pub divider: u8,
  pub reload_flag: bool,
}

impl APUSweep {
  pub fn new(ones_complement_negate: bool) -> Self {
    Self {
      enabled: false,
      shift_count: 0,
      negate: false,
      output: APUTimerRegister::new(),
      ones_complement_negate,
      divider_period: 0,
      divider: 0,
      reload_flag: false,
    }
  }

  pub fn tick(&mut self, timer: &APUTimerRegister) {
    let mut change_amount = (timer.timer() >> self.shift_count) as i16;
    if self.negate {
      change_amount = if self.ones_complement_negate {
        -change_amount - 1
      } else {
        -change_amount
      };
    }
    let target_period = timer.timer() as i16 + change_amount;
    let mute = timer.timer() < 8 || target_period > 0x7ff;

    // println!(
    //   "in: {}, enabled: {}, shift_count: {}, negate: {}, change_amount: {}, target_period: {}, mute: {}",
    //   timer.timer(),
    //   self.enabled,
    //   self.shift_count,
    //   self.negate,
    //   change_amount,
    //   target_period,
    //   mute
    // );

    if self.divider == 0 && self.enabled && !mute {
      self.output = timer.with_timer(target_period.clamp(0, i16::MAX) as u16);
      // println!("{}", self.output.timer());
    } else if self.divider == 0 || self.reload_flag {
      self.divider = self.divider_period;
      self.reload_flag = false;
      self.output = timer.clone();
      // println!("{}", self.output.timer());
    } else {
      self.divider -= 1;
    }
  }
}
