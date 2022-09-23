use anyhow::{anyhow, Result};
use audio_device::wasapi;
use audio_generator::{self as gen, Generator as _};

fn run_output<T>(client: wasapi::Client, mut config: wasapi::ClientConfig) -> Result<()>
where
    T: Copy + wasapi::Sample + core::Translate<f32>,
    [T]: rand::Fill,
{
    config.sample_rate = 120000;

    let initialized = client.initialize::<T>(config)?;
    let mut render_client = initialized.render_client()?;

    client.start()?;

    let config = initialized.config();
    let sample_rate = config.sample_rate as f32;

    dbg!(config);

    let mut a = gen::Sine::new(261.63, sample_rate);
    let mut b = gen::Sine::new(329.63, sample_rate);
    let mut c = gen::Sine::new(440.00, sample_rate);

    loop {
        let mut data = render_client.buffer_mut()?;

        for n in (0..data.len()).step_by(config.channels as usize) {
            let f = T::translate((a.sample() + b.sample() + c.sample()) * 0.01);

            for c in 0..config.channels as usize {
                data[n + c] = f;
            }
        }

        data.release()?;
    }
}

fn generate_audio() -> Result<()> {
    let output =
        wasapi::default_output_client()?.ok_or_else(|| anyhow!("no default device found"))?;
    let config = output.default_client_config()?;

    match config.sample_format {
        wasapi::SampleFormat::I16 => run_output::<i16>(output, config),
        wasapi::SampleFormat::F32 => run_output::<f32>(output, config),
    }
}

pub fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    println!("WARNING: This program will generate audio and we do our best to avoid them being too loud.");
    println!("Please make sure your volume is turned down!");
    println!();
    println!("Press [enter] to continue...");

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;

    let bg = ste::Builder::new().prelude(wasapi::audio_prelude).build()?;
    bg.submit(generate_audio)?;
    bg.join();
    Ok(())
}
