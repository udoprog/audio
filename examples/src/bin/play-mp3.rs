use anyhow::{anyhow, bail, Result};
use audio::BufMut;
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
    T: 'static + Send + cpal::Sample + audio::Sample + audio::Translate<f32>,
    f32: audio::Translate<i16>,
{
    let source = io::BufReader::new(fs::File::open(path)?);
    let decoder = minimp3::Decoder::new(source);

    let config = cpal::StreamConfig {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let pcm = audio::buf::Interleaved::new();
    let output = audio::buf::Interleaved::with_topology(config.channels as usize, 1024);
    let resample = audio::buf::Sequential::with_topology(config.channels as usize, CHUNK_SIZE);

    let mut writer = Writer {
        frames: 0,
        seconds: 0.0,
        decoder,
        pcm: audio::io::Read::new(pcm),
        resampler: None,
        output: audio::io::ReadWrite::empty(output),
        resample: audio::io::ReadWrite::empty(resample),
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
    pcm: audio::io::Read<audio::buf::Interleaved<i16>>,
    // The last mp3 frame decoded.
    last_frame: Option<minimp3::FrameInfo>,
    // Sampler that is used in case the sample rate of a decoded frame needs to
    // be resampled.
    resampler: Option<rubato::SincFixedIn<f32>>,
    // Output buffer to flush to device buffer.
    output: audio::io::ReadWrite<audio::buf::Interleaved<f32>>,
    // Resample buffer.
    resample: audio::io::ReadWrite<audio::buf::Sequential<f32>>,
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
    fn write_to<T>(&mut self, out: &mut [T]) -> anyhow::Result<()>
    where
        T: 'static + Send + audio::Sample + audio::Translate<f32>,
    {
        use audio::{io, wrap};
        use audio::{Buf, ExactSizeBuf, ReadBuf, WriteBuf};
        use rubato::Resampler;

        let mut data = io::Write::new(wrap::interleaved(out, self.device_channels));

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
            if let Some(frame) = &self.last_frame {
                if self.resample.remaining() == CHUNK_SIZE {
                    let device_sample_rate = self.device_sample_rate;

                    let resampler = self.resampler.get_or_insert_with(|| {
                        let params = InterpolationParameters {
                            sinc_len: 256,
                            f_cutoff: 0.95,
                            interpolation: InterpolationType::Linear,
                            oversampling_factor: 256,
                            window: WindowFunction::BlackmanHarris2,
                        };

                        let f_ratio = device_sample_rate as f64 / frame.sample_rate as f64;
                        SincFixedIn::<f32>::new(
                            f_ratio,
                            params,
                            CHUNK_SIZE,
                            frame.channels as usize,
                        )
                    });

                    resampler.process_with_buffer(
                        self.resample.as_ref(),
                        self.output.as_mut(),
                        &bittle::all(),
                    )?;

                    self.resample.clear();
                    let frames = self.output.as_ref().len();

                    self.output.set_read(0);
                    self.output.set_written(frames);
                    continue;
                }
            }

            // If we have information on a decoded frame, translate it into the
            // resample buffer until its filled up to its frames cap.
            if self.pcm.has_remaining() {
                io::translate_remaining(&mut self.pcm, &mut self.resample);
                continue;
            }

            let frame = self.decoder.next_frame_with_pcm(self.pcm.as_mut())?;
            self.pcm.set_read(0);
            self.resample.as_mut().resize_channels(frame.channels);

            // If the sample rate of the decoded frames matches the expected
            // output exactly, copy it directly to the output frame without
            // resampling.
            if frame.sample_rate as u32 == self.device_sample_rate {
                io::translate_remaining(&mut self.pcm, &mut self.output);
                continue;
            }

            self.last_frame = Some(frame);
        }

        // If the last decoded frame contains fewer channels than we have data
        // channels, copy channel 0 to the first two presumed stereo channels.
        if let Some(frame) = &self.last_frame {
            if frame.channels == 0 {
                bail!("tried to play stream with zero channels")
            }

            for to in frame.channels..usize::min(data.channels(), 2) {
                data.copy_channel(0, to);
            }
        }

        self.frames = self.frames.saturating_add(data.as_ref().frames());

        let seconds = self.frames as f32 / self.device_sample_rate as f32;
        let s = seconds.floor();

        if s > self.seconds {
            use std::io::Write as _;
            let mut o = std::io::stdout();
            write!(
                o,
                "\rTime: {:02}:{:02}",
                (s / 60.0) as u32,
                (s % 60.0) as u32
            )?;
            o.flush()?;
            self.seconds = seconds;
        }

        Ok(())
    }
}
