#[cfg(not(feature = "xaudio2"))]
pub fn main() {
    println!("xaudio2 support is not enabled");
}

#[cfg(feature = "xaudio2")]
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
