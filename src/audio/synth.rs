use std::ops::AddAssign;

use cpal::{traits::DeviceTrait, FromSample, Sample, SizedSample};
use smol::channel::{Sender, TryRecvError};

use super::{
  oscillator::{Oscillator, OscillatorCommand},
  stream_setup::StreamBuilder,
};

#[derive(Debug, PartialEq, Clone)]
pub enum SynthCommand {
  OscillatorCommand(usize, OscillatorCommand),
}

pub struct Synth {
  pub oscillators: Vec<Oscillator>,
}

impl StreamBuilder for Synth {
  type OutputType = (cpal::Stream, Sender<SynthCommand>);

  fn build_stream<SampleType: SizedSample + FromSample<f32> + AddAssign>(
    &self,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
  ) -> Result<Self::OutputType, anyhow::Error> {
    let num_channels = config.channels as usize;
    let sample_rate = config.sample_rate.0 as f32;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);
    let mut oscillators = self.oscillators.clone();

    let (sender, receiver) = smol::channel::unbounded::<SynthCommand>();

    let stream = device.build_output_stream(
      config,
      move |output: &mut [SampleType], _: &cpal::OutputCallbackInfo| {
        let command = match receiver.try_recv() {
          Ok(command) => Some(command),
          Err(recv_error) => match recv_error {
            TryRecvError::Empty => None,
            TryRecvError::Closed => panic!(),
          },
        };

        match command {
          Some(command) => {
            println!("Received {:?}", command);

            match command {
              SynthCommand::OscillatorCommand(index, command) => {
                oscillators[index].handle_command(command)
              }
            }
          }
          None => {}
        }

        process_frame(
          output,
          oscillators.as_mut_slice(),
          num_channels,
          sample_rate,
        )
      },
      err_fn,
      None,
    )?;

    Ok((stream, sender))
  }
}

fn process_frame<SampleType>(
  output: &mut [SampleType],
  oscillators: &mut [Oscillator],
  num_channels: usize,
  sample_rate: f32,
) where
  SampleType: Sample + FromSample<f32> + AddAssign,
{
  for frame in output.chunks_mut(num_channels) {
    let mut value: SampleType = SampleType::EQUILIBRIUM;
    for oscillator in &mut *oscillators {
      value += SampleType::from_sample(oscillator.tick(sample_rate));
    }

    // copy the same value to all channels
    for sample in frame.iter_mut() {
      *sample = value;
    }
  }
}
