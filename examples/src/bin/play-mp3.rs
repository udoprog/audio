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
        frames: 0,
        seconds: 0.0,
        decoder,
        pcm: minimp3::Pcm::new(),
        resampler: None,
        output: rotary::io::ReadWrite::new(output),
        resample: rotary::io::ReadWrite::new(resample),
        device_sample_rate: config.sample_rate.0,
        device_channels: config.channels as usize,
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
    // Frame counter.
    frames: usize,
    // Last second counter.
    seconds: f32,
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
    output: rotary::io::ReadWrite<rotary::Interleaved<f32>>,
    // Resample buffer.
    resample: rotary::io::ReadWrite<rotary::Sequential<f32>>,
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
        use rotary::{io, wrap};
        use rotary::{Buf as _, ExactSizeBuf as _, ReadBuf as _, WriteBuf as _};
        use rubato::Resampler;

        let mut data = wrap::interleaved(data, self.device_channels);
        let frames = data.remaining_mut();

        // Run the loop while there is buffer to fill.
        while data.has_remaining_mut() {
            // If there is output available, translate it to the data buffer
            // used by cpal.
            //
            // cpal's data buffer expects the output to be interleaved.
            if self.output.has_remaining() {
                io::translate_remaining(&mut self.output, &mut data);
                continue;
            }

            // If we have collected exactly one CHUNK_SIZE of resample buffer,
            // process it through the resampler and translate its result to the
            // output buffer.
            if self.resample.remaining() == CHUNK_SIZE {
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
                    self.resample.as_ref(),
                    self.output.as_mut(),
                    &bittle::all(),
                )?;

                self.resample.clear();
                let frames = self.output.as_ref().frames();

                self.output.set_read(0);
                self.output.set_written(frames);
                continue;
            }

            // If we have information on a decoded frame, translate it into the
            // resample buffer until its filled up to its frames cap.
            if let Some((frame, mut last_remaining)) = self.last_frame.take().filter(|p| p.1 > 0) {
                let mut pcm = wrap::interleaved(&self.pcm[..], frame.channels).tail(last_remaining);

                io::translate_remaining(&mut pcm, &mut self.resample);

                last_remaining = pcm.remaining();
                self.last_frame = Some((frame, last_remaining));
                continue;
            }

            let frame = self.decoder.next_frame_with_pcm(&mut self.pcm)?;
            self.resample.as_mut().resize_channels(frame.channels);

            let pcm = wrap::interleaved(&self.pcm[..], frame.channels);

            // If the sample rate of the decoded frames matches the expected
            // output exactly, copy it directly to the output frame without
            // resampling.
            if frame.sample_rate as u32 == self.device_sample_rate {
                io::translate_remaining(pcm, &mut self.output);
                continue;
            }

            self.last_frame = Some((frame, pcm.frames()));
        }

        self.frames += frames - data.remaining_mut();

        let seconds = self.frames as f32 / self.device_sample_rate as f32;

        if seconds.floor() > self.seconds {
            use std::io::Write as _;
            let mut o = std::io::stdout();
            write!(o, "\r{}", seconds.floor())?;
            o.flush()?;
            self.seconds = seconds;
        }

        Ok(())
    }
}
