use cpal::traits::{DeviceTrait, HostTrait};

pub trait StreamSpawner {
  type OutputType;

  fn spawn_stream<
    SampleType: cpal::SizedSample
      + cpal::FromSample<f32>
      + core::iter::Sum<SampleType>
      + core::ops::Add<SampleType, Output = SampleType>,
  >(
    &self,
    device: cpal::Device,
    config: &cpal::StreamConfig,
  ) -> Result<Self::OutputType, anyhow::Error>;
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
  let host = cpal::default_host();

  let device = host
    .default_output_device()
    .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
  println!("Output device : {}", device.name()?);

  let config = device.default_output_config()?;
  println!("Default output config : {:?}", config);

  Ok((host, device, config))
}

pub fn stream_setup_for<B: StreamSpawner<OutputType = T>, T>(
  spawner: B,
) -> Result<T, anyhow::Error>
where
{
  let (_host, device, config) = host_device_setup()?;

  match config.sample_format() {
    cpal::SampleFormat::I8 => spawner.spawn_stream::<i8>(device, &config.into()),
    cpal::SampleFormat::I16 => spawner.spawn_stream::<i16>(device, &config.into()),
    cpal::SampleFormat::I32 => spawner.spawn_stream::<i32>(device, &config.into()),
    cpal::SampleFormat::I64 => spawner.spawn_stream::<i64>(device, &config.into()),
    cpal::SampleFormat::U8 => spawner.spawn_stream::<u8>(device, &config.into()),
    cpal::SampleFormat::U16 => spawner.spawn_stream::<u16>(device, &config.into()),
    cpal::SampleFormat::U32 => spawner.spawn_stream::<u32>(device, &config.into()),
    cpal::SampleFormat::U64 => spawner.spawn_stream::<u64>(device, &config.into()),
    cpal::SampleFormat::F32 => spawner.spawn_stream::<f32>(device, &config.into()),
    cpal::SampleFormat::F64 => spawner.spawn_stream::<f64>(device, &config.into()),
    sample_format => Err(anyhow::Error::msg(format!(
      "Unsupported sample format '{sample_format}'"
    ))),
  }
}
