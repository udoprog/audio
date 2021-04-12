#[cfg(not(all(windows, feature = "wasapi")))]
pub fn main() -> anyhow::Result<()> {
    println!("wasapi support is not enabled");
    Ok(())
}

#[cfg(all(windows, feature = "wasapi"))]
fn main() -> anyhow::Result<()> {
    self::wasapi::main()
}

#[cfg(all(windows, feature = "wasapi"))]
mod wasapi {
    use anyhow::{anyhow, Result};
    use audio_device::driver::events::Handle;
    use audio_device::wasapi;
    use audio_generator::{self as gen, Generator as _};

    async fn run_output<T>(
        handle: &Handle,
        client: wasapi::Client,
        mut config: wasapi::ClientConfig,
    ) -> Result<()>
    where
        T: Copy + wasapi::Sample + audio_core::Translate<f32>,
        [T]: rand::Fill,
    {
        config.sample_rate = 120000;

        let initialized = client.initialize_async::<T>(handle, config)?;
        let mut render_client = initialized.render_client()?;

        client.start()?;

        let config = initialized.config();
        let sample_rate = config.sample_rate as f32;

        dbg!(config);

        let mut a = gen::Sin::new(261.63, sample_rate);
        let mut e = gen::Sin::new(329.63, sample_rate);
        let mut b = gen::Sin::new(440.00, sample_rate);

        loop {
            let mut data = render_client.buffer_mut_async().await?;

            for n in (0..data.len()).step_by(config.channels as usize) {
                let f = T::translate((a.sample() + b.sample() + e.sample()) * 0.01);

                for c in 0..config.channels as usize {
                    data[n + c] = f;
                }
            }

            data.release()?;
        }
    }

    #[tokio::main]
    pub async fn main() -> Result<()> {
        println!("WARNING: This program will generate audio and we do our best to avoid them being too loud.");
        println!("Please make sure your volume is turned down!");
        println!();
        println!("Press [enter] to continue...");

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;

        let handle = Handle::new()?;
        let audio_thread = ste::Builder::new().prelude(wasapi::audio_prelude).build()?;

        audio_thread
            .submit_async(async {
                let output = wasapi::default_output_client()?
                    .ok_or_else(|| anyhow!("no default device found"))?;

                let config = output.default_client_config()?;

                match config.sample_format {
                    wasapi::SampleFormat::I16 => {
                        run_output::<i16>(&handle, output, config).await?;
                    }
                    wasapi::SampleFormat::F32 => {
                        run_output::<f32>(&handle, output, config).await?;
                    }
                }

                Ok::<(), anyhow::Error>(())
            })
            .await??;

        audio_thread.join()?;
        Ok(())
    }
}
