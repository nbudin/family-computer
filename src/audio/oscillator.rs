use std::{any::Any, f32::consts::PI};

use super::audio_channel::AudioChannel;

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(dead_code)]
pub enum Waveform {
  Sine,
  Square,
  Saw,
  Triangle,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum OscillatorCommand {
  SetWaveform(Waveform),
  SetFrequency(f32),
  SetAmplitude(f32),
  SetDutyCycle(f32),
}

#[derive(Clone)]
pub struct Oscillator {
  pub waveform: Waveform,
  pub current_sample_index: f32,
  pub frequency_hz: f32,
  pub amplitude: f32,
  pub duty_cycle: f32,
}

impl AudioChannel for Oscillator {
  fn get_next_sample(&mut self, sample_rate: f32) -> f32 {
    match self.waveform {
      Waveform::Sine => self.sine_wave(sample_rate),
      Waveform::Square => self.square_wave(sample_rate),
      Waveform::Saw => self.saw_wave(sample_rate),
      Waveform::Triangle => self.triangle_wave(sample_rate),
    }
  }

  fn handle_command(&mut self, command: Box<dyn Any + Send + Sync>) {
    let Ok(command) = command.downcast::<OscillatorCommand>() else {
      return;
    };

    match command.as_ref() {
      OscillatorCommand::SetWaveform(waveform) => self.set_waveform(waveform.clone()),
      OscillatorCommand::SetFrequency(frequency) => self.set_frequency(frequency.clone()),
      OscillatorCommand::SetAmplitude(amplitude) => self.set_amplitude(amplitude.clone()),
      OscillatorCommand::SetDutyCycle(duty_cycle) => self.set_duty_cycle(duty_cycle.clone()),
    }
  }
}

impl Oscillator {
  pub fn advance_sample(&mut self, sample_rate: f32) {
    self.current_sample_index = (self.current_sample_index + 1.0) % sample_rate;
  }

  pub fn set_waveform(&mut self, waveform: Waveform) {
    self.waveform = waveform;
  }

  pub fn set_frequency(&mut self, frequency: f32) {
    self.frequency_hz = frequency;
  }

  pub fn set_amplitude(&mut self, amplitude: f32) {
    self.amplitude = amplitude;
  }

  pub fn set_duty_cycle(&mut self, duty_cycle: f32) {
    self.duty_cycle = duty_cycle;
  }

  fn calculate_sine_output_from_freq(&self, freq: f32, sample_rate: f32) -> f32 {
    let two_pi = 2.0 * std::f32::consts::PI;
    (self.current_sample_index * freq * two_pi / sample_rate).sin()
  }

  fn is_multiple_of_freq_above_nyquist(&self, multiple: f32, sample_rate: f32) -> bool {
    self.frequency_hz * multiple > sample_rate / 2.0
  }

  fn sine_wave(&mut self, sample_rate: f32) -> f32 {
    self.advance_sample(sample_rate);
    self.calculate_sine_output_from_freq(self.frequency_hz, sample_rate) * self.amplitude
  }

  fn generative_waveform(
    &mut self,
    harmonic_index_increment: i32,
    gain_exponent: f32,
    sample_rate: f32,
  ) -> f32 {
    self.advance_sample(sample_rate);
    let mut i = 1;
    let mut a = 0.0;
    let mut b = 0.0;
    let p = self.duty_cycle * 2.0 * PI;

    while !self.is_multiple_of_freq_above_nyquist(i as f32, sample_rate) {
      let gain = 1.0 / (i as f32).powf(gain_exponent);
      a += gain * self.calculate_sine_output_from_freq(self.frequency_hz * i as f32, sample_rate);
      b += gain
        * self.calculate_sine_output_from_freq(
          self.frequency_hz * i as f32 - (p * i as f32),
          sample_rate,
        );
      i += harmonic_index_increment;
    }
    (a - b) * self.amplitude
  }

  fn square_wave(&mut self, sample_rate: f32) -> f32 {
    self.generative_waveform(2, 1.0, sample_rate)
  }

  fn saw_wave(&mut self, sample_rate: f32) -> f32 {
    self.generative_waveform(1, 1.0, sample_rate)
  }

  fn triangle_wave(&mut self, sample_rate: f32) -> f32 {
    self.generative_waveform(2, 2.0, sample_rate)
  }
}