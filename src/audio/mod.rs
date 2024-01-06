pub mod oscillator;
mod stream_setup;
mod synth;

use cpal::traits::StreamTrait;

use crate::audio::{
  oscillator::{Oscillator, Waveform},
  stream_setup::stream_setup_for,
  synth::{Synth, SynthCommand},
};

use self::oscillator::OscillatorCommand;

pub fn audio_test() -> Result<(), anyhow::Error> {
  let synth = Synth {
    oscillators: vec![
      Oscillator {
        waveform: Waveform::Sine,
        current_sample_index: 0.0,
        frequency_hz: 261.63,
        amplitude: 0.1,
      },
      Oscillator {
        waveform: Waveform::Sine,
        current_sample_index: 0.0,
        frequency_hz: 329.63,
        amplitude: 0.0,
      },
      Oscillator {
        waveform: Waveform::Sine,
        current_sample_index: 0.0,
        frequency_hz: 392.0,
        amplitude: 0.0,
      },
    ],
  };

  let (stream, sender) = stream_setup_for(synth)?;
  stream.play()?;

  let time_at_start = std::time::Instant::now();
  println!("Time at start: {:?}", time_at_start);

  let mut last_command: Option<SynthCommand> = None;

  loop {
    let time_since_start = std::time::Instant::now()
      .duration_since(time_at_start)
      .as_secs_f32();

    let command_to_send: Option<SynthCommand>;

    if time_since_start < 1.0 {
      command_to_send = Some(SynthCommand::OscillatorCommand(
        0,
        OscillatorCommand::SetFrequency(261.63),
      ));
    } else if time_since_start < 3.0 {
      command_to_send = Some(SynthCommand::OscillatorCommand(
        1,
        OscillatorCommand::SetAmplitude(0.1),
      ));
    } else if time_since_start < 5.0 {
      command_to_send = Some(SynthCommand::OscillatorCommand(
        2,
        OscillatorCommand::SetAmplitude(0.1),
      ));
    } else {
      break;
    }

    if last_command != command_to_send {
      if let Some(command_to_send) = command_to_send {
        sender.send_blocking(command_to_send.clone()).unwrap();
        last_command = Some(command_to_send);
      }
    }
  }

  Ok(())
}
