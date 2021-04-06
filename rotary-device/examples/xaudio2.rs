use anyhow::{anyhow, Result};

pub fn main() -> Result<()> {
    let audio_thread = rotary_device::AudioThread::new()?;

    audio_thread.submit(|| {
        let output = rotary_device::xaudio2::default_audio()?;
        let output = output.ok_or_else(|| anyhow!("no default output device found"))?;

        Ok::<(), anyhow::Error>(())
    })??;

    audio_thread.join()?;
    Ok(())
}
