use std::{any::Any, collections::HashMap, fmt::Debug, hash::Hash};

use cpal::{
  traits::{DeviceTrait, StreamTrait},
  FromSample, Sample,
};
use smol::channel::{Sender, TryRecvError};

use super::{audio_channel::AudioChannel, stream_setup::StreamSpawner};

#[derive(Debug)]
pub enum SynthCommand<ChannelIdentifier: Clone + Eq + PartialEq + Hash + Debug + Send> {
  ChannelCommand(ChannelIdentifier, Box<dyn Any + Send + Sync>),
}

pub struct Synth<ChannelIdentifier: Clone + Eq + PartialEq + Hash + Debug + Send> {
  pub channels: HashMap<ChannelIdentifier, Box<dyn AudioChannel>>,
}

impl<ChannelIdentifier: Clone + Eq + PartialEq + Hash + Debug + Send + 'static> StreamSpawner
  for Synth<ChannelIdentifier>
{
  type OutputType = Sender<SynthCommand<ChannelIdentifier>>;

  fn spawn_stream<
    SampleType: cpal::SizedSample
      + cpal::FromSample<f32>
      + core::iter::Sum<SampleType>
      + core::ops::Add<SampleType, Output = SampleType>,
  >(
    &self,
    device: cpal::Device,
    config: &cpal::StreamConfig,
  ) -> Result<Self::OutputType, anyhow::Error> {
    let num_channels = config.channels as usize;
    let sample_rate = config.sample_rate.0 as f32;
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);
    let mut channels = self
      .channels
      .iter()
      .map(|(id, channel)| (id.clone(), dyn_clone::clone_box(channel)))
      .collect::<HashMap<_, _>>();
    let config = config.clone();

    let (sender, receiver) = smol::channel::unbounded::<SynthCommand<ChannelIdentifier>>();

    std::thread::spawn(move || {
      let stream = device
        .build_output_stream(
          &config,
          move |output: &mut [SampleType], _: &cpal::OutputCallbackInfo| {
            let command = match receiver.try_recv() {
              Ok(command) => Some(command),
              Err(recv_error) => match recv_error {
                TryRecvError::Empty => None,
                TryRecvError::Closed => panic!(),
              },
            };

            match command {
              Some(command) => match command {
                SynthCommand::ChannelCommand(index, command) => {
                  channels.get_mut(&index).unwrap().handle_command(command)
                }
              },
              None => {}
            }

            process_frame(
              output,
              channels
                .iter_mut()
                .map(|(_identifier, channel)| channel.as_mut())
                .collect(),
              num_channels,
              sample_rate,
            )
          },
          err_fn,
          None,
        )
        .unwrap();

      stream.play().unwrap();

      std::thread::park();
    });

    Ok(sender)
  }
}

fn process_frame<'a, SampleType>(
  output: &mut [SampleType],
  mut channels: Vec<&mut Box<dyn AudioChannel>>,
  num_channels: usize,
  sample_rate: f32,
) where
  SampleType: Sample
    + FromSample<f32>
    + core::iter::Sum<SampleType>
    + core::ops::Add<SampleType, Output = SampleType>,
{
  let amplitude_divisor = channels.len() as f32;
  for frame in output.chunks_mut(num_channels) {
    let value: SampleType = SampleType::EQUILIBRIUM
      + channels
        .iter_mut()
        .map(|channel| {
          SampleType::from_sample(channel.get_next_sample(sample_rate) / amplitude_divisor)
        })
        .sum::<SampleType>();

    // copy the same value to all channels
    for sample in frame.iter_mut() {
      *sample = value;
    }
  }
}
