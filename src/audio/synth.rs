use std::{
  any::Any,
  collections::{HashMap, VecDeque},
  fmt::Debug,
  hash::Hash,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::Duration,
};

use cpal::{
  traits::{DeviceTrait, StreamTrait},
  FromSample, Sample, StreamInstant,
};
use smol::channel::{Sender, TryRecvError};

use super::{audio_channel::AudioChannel, stream_setup::StreamSpawner};

const MIXER_AMPLITUDE: f32 = 15.0;

#[derive(Debug)]
pub enum SynthCommand<ChannelIdentifier: Clone + Eq + PartialEq + Hash + Debug + Send> {
  ChannelCommand(ChannelIdentifier, Box<dyn Any + Send + Sync>, Duration),
}

impl<ChannelIdentifier: Clone + Eq + PartialEq + Hash + Debug + Send>
  SynthCommand<ChannelIdentifier>
{
  pub fn time(&self) -> Duration {
    match self {
      SynthCommand::ChannelCommand(_, _, time) => time.to_owned(),
    }
  }
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
      let mut start_time: Option<StreamInstant> = None;
      let mut command_queue: VecDeque<SynthCommand<ChannelIdentifier>> =
        VecDeque::with_capacity(32);
      let mut last_channel_recv: Option<StreamInstant> = None;
      let receive_interval = Duration::from_millis(2);
      let shutdown = Arc::new(AtomicBool::new(false));
      let shutdown_sender = shutdown.clone();
      let control_thread = std::thread::current();

      let stream = device
        .build_output_stream(
          &config,
          move |output: &mut [SampleType], callback_info: &cpal::OutputCallbackInfo| {
            let timestamp = callback_info.timestamp();
            let should_receive = last_channel_recv.is_none()
              || last_channel_recv.is_some_and(|last_channel_recv| {
                timestamp.callback.duration_since(&last_channel_recv) > Some(receive_interval)
              });

            if should_receive {
              last_channel_recv = Some(timestamp.callback);
              loop {
                let command = match receiver.try_recv() {
                  Ok(command) => command,
                  Err(recv_error) => match recv_error {
                    TryRecvError::Empty => {
                      break;
                    }
                    TryRecvError::Closed => {
                      shutdown_sender.store(true, Ordering::Relaxed);
                      control_thread.unpark();
                      println!("Shutting down control thread");
                      break;
                    }
                  },
                };

                command_queue.push_back(command);
              }
            }

            let mut commands_to_run_now: Vec<SynthCommand<ChannelIdentifier>> = Vec::new();

            let playback_time_since_start = start_time.map(|start_time| {
              timestamp
                .playback
                .duration_since(&start_time)
                .unwrap_or_default()
            });

            if let Some(playback_time_since_start) = playback_time_since_start {
              loop {
                let command = command_queue.pop_front();

                if let Some(command) = command {
                  if command.time() > playback_time_since_start {
                    command_queue.push_front(command);
                    break;
                  } else {
                    commands_to_run_now.push(command);
                  }
                } else {
                  break;
                }
              }
            } else {
              start_time = Some(timestamp.playback);
            }

            for command in commands_to_run_now {
              match command {
                SynthCommand::ChannelCommand(index, command, _) => {
                  channels.get_mut(&index).unwrap().handle_command(command)
                }
              }
            }

            process_frame(
              output,
              channels
                .iter_mut()
                .map(|(_identifier, channel)| channel.as_mut())
                .collect(),
              num_channels,
              sample_rate,
              playback_time_since_start.unwrap_or_default(),
            )
          },
          err_fn,
          None,
        )
        .unwrap();

      stream.play().unwrap();

      while !shutdown.load(Ordering::Relaxed) {
        std::thread::park();
      }

      println!("Audio thread received shutdown signal");
    });

    Ok(sender)
  }
}

fn process_frame<SampleType>(
  output: &mut [SampleType],
  mut channels: Vec<&mut Box<dyn AudioChannel>>,
  num_channels: usize,
  sample_rate: f32,
  timestamp: Duration,
) where
  SampleType: Sample
    + FromSample<f32>
    + core::iter::Sum<SampleType>
    + core::ops::Add<SampleType, Output = SampleType>,
{
  for frame in output.chunks_mut(num_channels) {
    let value: SampleType = SampleType::EQUILIBRIUM
      + channels
        .iter_mut()
        .map(|channel| {
          let channel_amplitude = channel.mix_amplitude();
          let f32_value = channel.get_next_sample(sample_rate, timestamp) * channel_amplitude;
          SampleType::from_sample(f32_value * MIXER_AMPLITUDE)
        })
        .sum::<SampleType>();

    // copy the same value to all output channels
    for sample in frame.iter_mut() {
      *sample = value
    }
  }
}
