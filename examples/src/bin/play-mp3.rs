use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{InterpolationParameters, InterpolationType, SincFixedIn, WindowFunction};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CHUNK_SIZE: usize = 1024;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os();
    args.next();

    let path = PathBuf::from(args.next().expect("missing argument <path>"));

    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("failed to build default device"))?;

    let config = device.default_output_config()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&path, &device, &config),
        cpal::SampleFormat::I16 => run::<i16>(&path, &device, &config),
        cpal::SampleFormat::U16 => run::<u16>(&path, &device, &config),
    }
}

fn run<T>(path: &Path, device: &cpal::Device, config: &cpal::SupportedStreamConfig) -> Result<()>
where
    T: 'static + Send + cpal::Sample + rotary::Sample + rotary::Translate<f32>,
    f32: rotary::Translate<i16>,
{
    use rotary::BufMut as _;

    let source = io::BufReader::new(fs::File::open(path)?);
    let decoder = minimp3::Decoder::new(source);

    let config = cpal::StreamConfig {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let output = rotary::Interleaved::with_topology(config.channels as usize, 1024);
    let resample = rotary::Sequential::with_topology(config.channels as usize, CHUNK_SIZE);

    let mut writer = Writer {
        decoder,
        pcm: minimp3::Pcm::new(),
        resampler: None,
        output: output.read_write(),
        resample,
        device_sample_rate: config.sample_rate.0,
        device_channels: config.channels as usize,
        resample_avail: 0,
        last_frame: None,
    };

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Err(e) = writer.write_to(data) {
                println!("failed to write data: {}", e);
            }
        },
        move |err| {
            println!("audio output error: {}", err);
        },
    )?;

    stream.play()?;

    let mut line = String::new();

    println!("press [enter] to shut down");
    std::io::stdin().read_line(&mut line)?;
    Ok(())
}

struct Writer<R>
where
    R: io::Read,
{
    // The open mp3 decoder.
    decoder: minimp3::Decoder<R>,
    // Buffer used for mp3 decoding.
    pcm: minimp3::Pcm,
    // The last mp3 frame decoded.
    last_frame: Option<(minimp3::FrameInfo, usize)>,
    // Sampler that is used in case the sample rate of a decoded frame needs to
    // be resampled.
    resampler: Option<rubato::SincFixedIn<f32>>,
    // Output buffer to flush to device buffer.
    output: rotary::buf::ReadWrite<rotary::Interleaved<f32>>,
    // Resample buffer.
    resample: rotary::Sequential<f32>,
    /// Frames available in the resample buffer.
    resample_avail: usize,
    // Sample rate expected to be written to the device.
    device_sample_rate: u32,
    // Number of channels in the device.
    device_channels: usize,
}

impl<R> Writer<R>
where
    R: io::Read,
{
    // The decoder loop.
    fn write_to<T>(&mut self, data: &mut [T]) -> anyhow::Result<()>
    where
        T: 'static + Send + rotary::Sample + rotary::Translate<f32>,
    {
        use rotary::{Buf as _, BufMut as _};
        use rubato::Resampler;

        let mut data = rotary::wrap::interleaved(data, self.device_channels).writer();

        // Run the loop while there is buffer to fill.
        while data.has_remaining_mut() {
            // If there is output available, translate it to the data buffer
            // used by cpal.
            //
            // cpal's data buffer expects the output to be interleaved.
            if self.output.has_remaining() {
                data.translate(self.output.read());
                continue;
            }

            // If we have collected exactly one CHUNK_SIZE of resample buffer,
            // process it through the resampler and translate its result to the
            // output buffer.
            if self.resample_avail == CHUNK_SIZE {
                let device_sample_rate = self.device_sample_rate;

                let (frame, _) = self.last_frame.as_ref().unwrap();

                let resampler = self.resampler.get_or_insert_with(|| {
                    let params = InterpolationParameters {
                        sinc_len: 256,
                        f_cutoff: 0.95,
                        interpolation: InterpolationType::Linear,
                        oversampling_factor: 256,
                        window: WindowFunction::BlackmanHarris2,
                    };

                    let f_ratio = device_sample_rate as f64 / frame.sample_rate as f64;
                    SincFixedIn::<f32>::new(f_ratio, params, CHUNK_SIZE, frame.channels as usize)
                });

                resampler.process_with_buffer(
                    &mut self.resample,
                    &mut self.output.as_mut(),
                    &rotary::mask::all(),
                )?;

                self.resample_avail = 0;
                let frames = self.output.as_ref().frames();
                self.output.set_written(frames);
                continue;
            }

            // If we have information on a decoded frame, translate it into the
            // resample buffer until its filled up to its frames cap.
            if let Some((frame, mut pcm_avail)) = self.last_frame.take().filter(|p| p.1 > 0) {
                let pcm = rotary::wrap::interleaved(&self.pcm[..], frame.channels);

                let from = pcm.tail(pcm_avail);
                let to = (&mut self.resample).skip(self.resample_avail);

                let copied = usize::min(from.frames(), to.frames());

                rotary::utils::translate(from, to);

                pcm_avail -= copied;
                self.resample_avail += copied;

                self.last_frame = Some((frame, pcm_avail));
                continue;
            }

            let frame = self.decoder.next_frame_with_pcm(&mut self.pcm)?;
            self.resample.resize_channels(frame.channels);

            let pcm = rotary::wrap::interleaved(&self.pcm[..], frame.channels);

            // If the sample rate of the decoded frames matches the expected
            // output exactly, copy it directly to the output frame without
            // resampling.
            if frame.sample_rate as u32 == self.device_sample_rate {
                self.output.translate(pcm);
                continue;
            }

            self.last_frame = Some((frame, pcm.frames()));
        }

        Ok(())
    }
}
